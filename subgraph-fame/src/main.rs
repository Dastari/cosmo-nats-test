use std::sync::Arc;
use async_graphql::{http::GraphiQLSource, Object, Schema, SimpleObject, Subscription, ID};
use async_graphql::futures_util::Stream;
use async_graphql_axum::{GraphQLRequest, GraphQLResponse, GraphQLSubscription};
use async_nats::Client as NatsClient;
use axum::{extract::State, http::Method, response::Html, routing::{get, get_service, post}, Router};
use clap::Parser;
use tokio::sync::{broadcast, RwLock};
use tower_http::cors::{Any, CorsLayer};

type Storage = Arc<RwLock<i32>>;
type Sender = Arc<broadcast::Sender<i32>>;
type NatsPool = Arc<RwLock<Option<NatsClient>>>;
type NatsUrl = Arc<String>;

#[derive(SimpleObject)]
#[graphql(shareable)]
struct Subgraph {
    id: ID,
    value: Option<i32>,
}

struct Query {
    owner_id: ID,
    storage: Storage,
}

struct Mutation {
    owner_id: ID,
    storage: Storage,
    sender: Sender,
    nats_pool: NatsPool,
    nats_url: NatsUrl,
    subject: String,
}

struct Subscriptions {
    sender: Sender,
}

#[Object(name = "Query")]
impl Query {
    #[graphql(name = "FameQueryValue")]
    async fn query_value(&self) -> i32 {
        *self.storage.read().await
    }

    #[graphql(entity)]
    async fn subgraph_by_id(&self, id: ID) -> Subgraph {
        let v = *self.storage.read().await;
        Subgraph { id, value: Some(v) }
    }
}

#[Object(name = "Mutation")]
impl Mutation {
    #[graphql(name = "FameIncrementValue")]
    async fn increment_vlue(&self, by: Option<i32>) -> i32 {
        let inc = by.unwrap_or(1);
        let mut w = self.storage.write().await;
        *w += inc;
        let new_val = *w;
        let _ = self.sender.send(new_val);
        let payload = format!(r#"{{"id":"{}","value":{}}}"#, self.owner_id.to_string(), new_val).into_bytes();
        nats_publish(self.nats_pool.clone(), self.nats_url.clone(), self.subject.clone(), payload).await;
        new_val
    }
}

#[Subscription(name = "Subscription")]
impl Subscriptions {
    #[graphql(name = "FameOnValueChange")]
    async fn on_value_change(&self) -> impl Stream<Item = i32> {
        let mut rx = self.sender.subscribe();
        async_stream::stream! {
            while let Ok(v) = rx.recv().await {
                yield v;
            }
        }
    }
}

async fn nats_publish(pool: NatsPool, url: NatsUrl, subject: String, payload: Vec<u8>) {
    let try_publish = |c: NatsClient, s: String, p: Vec<u8>| async move { c.publish(s, p.into()).await };
    if let Some(c) = pool.read().await.as_ref() {
        if try_publish(c.clone(), subject.clone(), payload.clone()).await.is_ok() {
            return;
        }
    }
    if let Ok(client) = async_nats::connect(url.as_str()).await {
        {
            let mut w = pool.write().await;
            *w = Some(client.clone());
        }
        let _ = try_publish(client, subject, payload).await;
    }
}

type AppSchema = Schema<Query, Mutation, Subscriptions>;

async fn graphql_handler(State(schema): State<AppSchema>, req: GraphQLRequest) -> GraphQLResponse {
    schema.execute(req.into_inner()).await.into()
}

async fn graphiql(title: String) -> Html<String> {
    Html(
        GraphiQLSource::build()
            .title(&title)
            .endpoint("/graphql")
            .subscription_endpoint("/graphql")
            .finish(),
    )
}

#[derive(Parser, Debug, Clone)]
struct Opts {
    #[arg(long, default_value_t = 9001)]
    port: u16,
    #[arg(long, env = "NATS_URL", default_value = "nats://127.0.0.1:4222")]
    nats_url: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let opts = Opts::parse();

    let storage: Storage = Arc::new(RwLock::new(0));
    let (tx, _) = broadcast::channel::<i32>(100);
    let sender: Sender = Arc::new(tx);

    let nats_pool: NatsPool = Arc::new(RwLock::new(None));
    let nats_url: NatsUrl = Arc::new(opts.nats_url.clone());
    let subject = "gema.fame.value.updated".to_string();
    let owner_id = ID::from("fame");

    let schema = Schema::build(
        Query { owner_id: owner_id.clone(), storage: storage.clone() },
        Mutation {
            owner_id: owner_id.clone(),
            storage: storage.clone(),
            sender: sender.clone(),
            nats_pool: nats_pool.clone(),
            nats_url: nats_url.clone(),
            subject: subject.clone(),
        },
        Subscriptions { sender: sender.clone() },
    )
    .enable_federation()
    .finish();

    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST])
        .allow_headers(Any)
        .allow_origin(Any);

    let app = Router::new()
        .route("/", get(|| graphiql("Fame".to_string())))
        .route("/graphql", post(graphql_handler))
        .route("/graphql", get_service(GraphQLSubscription::new(schema.clone())))
        .layer(cors)
        .with_state(schema);

    let listener = tokio::net::TcpListener::bind(("127.0.0.1", opts.port)).await?;
    println!("http://127.0.0.1:{}/", opts.port);
    println!("/graphql");
    axum::serve(listener, app).await?;
    Ok(())
}
