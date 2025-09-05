use async_graphql::{
    Context, Object, Schema, SimpleObject, Subscription, ID,
};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse, GraphQLSubscription};
use axum::{
    extract::State,
    http::Method,
    routing::{post, get_service},
    Router,
};
use futures_util::Stream;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::{broadcast, RwLock};
use tower_http::cors::{Any, CorsLayer};
// use tower::util::ServiceExt;

type Storage = Arc<RwLock<HashMap<String, i32>>>;
type CounterSender = Arc<broadcast::Sender<i32>>;

#[derive(SimpleObject)]
#[graphql(shareable)]
struct Endpoint {
    id: ID,
    subgraph2_count: Option<i32>,
}

impl Endpoint {
    async fn find_by_id(id: ID, ctx: &Context<'_>) -> Self {
        let storage = ctx.data::<Storage>().unwrap();
        let data = storage.read().await;
        let count = data.get(&format!("endpoint_{}", id.as_str())).unwrap_or(&100);
        
        Self {
            id: id.clone(),
            subgraph2_count: Some(*count),
        }
    }
}

struct Query;

#[Object]
impl Query {
    async fn subgraph2_query_value(&self, ctx: &Context<'_>) -> i32 {
        let storage = ctx.data::<Storage>().unwrap();
        let data = storage.read().await;
        *data.get("counter").unwrap_or(&0)
    }

    #[graphql(entity)]
    async fn endpoint_by_id(&self, ctx: &Context<'_>, id: ID) -> Endpoint {
        Endpoint::find_by_id(id, ctx).await
    }
}

struct Mutation;

#[Object]
impl Mutation {
    async fn subgraph2_increment_value(&self, ctx: &Context<'_>, by: Option<i32>) -> i32 {
        let storage = ctx.data::<Storage>().unwrap();
        let sender = ctx.data::<CounterSender>().unwrap();
        
        let mut data = storage.write().await;
        let current = data.get("counter").unwrap_or(&0);
        let new_value = current + by.unwrap_or(1);
        data.insert("counter".to_string(), new_value);
        
        // Broadcast the new value to subscribers
        let _ = sender.send(new_value);
        
        new_value
    }
}

struct Subscription;

#[Subscription]
impl Subscription {
    async fn subgraph2_on_change_value(&self, ctx: &Context<'_>) -> impl Stream<Item = i32> {
        let sender = ctx.data::<CounterSender>().unwrap();
        let mut receiver = sender.subscribe();
        
        async_stream::stream! {
            while let Ok(value) = receiver.recv().await {
                yield value;
            }
        }
    }
}

type Subgraph2Schema = Schema<Query, Mutation, Subscription>;

async fn graphql_handler(
    State(schema): State<Subgraph2Schema>,
    req: GraphQLRequest,
) -> GraphQLResponse {
    schema.execute(req.into_inner()).await.into()
}

// WebSocket subscriptions are handled via get_service(GraphQLSubscription::new(schema))



#[tokio::main]
async fn main() {
    let storage: Storage = Arc::new(RwLock::new(HashMap::new()));
    let (sender, _) = broadcast::channel::<i32>(100);
    let counter_sender = Arc::new(sender);

    let schema = Schema::build(Query, Mutation, Subscription)
        .enable_federation()
        .data(storage)
        .data(counter_sender)
        .finish();

    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST])
        .allow_headers(Any)
        .allow_origin(Any);

    let app = Router::new()
        .route("/graphql", post(graphql_handler))
        .route("/graphql", get_service(GraphQLSubscription::new(schema.clone())))
        .layer(cors)
        .with_state(schema);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8083")
        .await
        .unwrap();

    println!("🦀 Subgraph-2 listening (HTTP+WS) on http://127.0.0.1:8083/graphql");

    axum::serve(listener, app).await.unwrap();
}