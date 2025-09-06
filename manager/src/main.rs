use async_graphql::{http::GraphiQLSource, EmptyMutation, EmptySubscription, Object, Schema, SimpleObject, ID};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{extract::State, http::Method, response::Html, routing::get, routing::post, Router};
use tower_http::cors::{Any, CorsLayer};

#[derive(SimpleObject)]
#[graphql(shareable)]
struct Subgraph {
    id: ID,
}

struct Query;

#[Object]
impl Query {
    async fn ping(&self) -> &str {
        "ok"
    }

    #[graphql(entity)]
    async fn subgraph_by_id(&self, id: ID) -> Subgraph {
        Subgraph { id }
    }
}

type AppSchema = Schema<Query, EmptyMutation, EmptySubscription>;

async fn graphql_handler(State(schema): State<AppSchema>, req: GraphQLRequest) -> GraphQLResponse {
    schema.execute(req.into_inner()).await.into()
}

async fn graphiql() -> Html<String> {
    Html(
        GraphiQLSource::build()
            .title("Subgraph Owner")
            .endpoint("/graphql")
            .finish(),
    )
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let schema = Schema::build(Query, EmptyMutation, EmptySubscription)
        .enable_federation()
        .finish();

    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST])
        .allow_headers(Any)
        .allow_origin(Any);

    let app = Router::new()
        .route("/", get(graphiql))
        .route("/graphql", post(graphql_handler))
        .layer(cors)
        .with_state(schema);

    let listener = tokio::net::TcpListener::bind(("127.0.0.1", 9000)).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
