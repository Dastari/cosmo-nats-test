use std::sync::Arc;
use async_graphql::http::GraphiQLSource;
use async_graphql::dynamic::{
    Field, FieldFuture, FieldValue, InputValue, Object, Schema, Subscription, SubscriptionField,
    SubscriptionFieldFuture, TypeRef,
};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse, GraphQLSubscription};
use async_nats::Client as NatsClient;
use axum::{
    extract::State,
    http::Method,
    response::Html,
    routing::{get, get_service, post},
    Router,
};
use clap::Parser;
use tokio::sync::{broadcast, RwLock};
use tower_http::cors::{Any, CorsLayer};

type Storage = Arc<RwLock<i32>>;
type Sender = Arc<broadcast::Sender<i32>>;
type NatsPool = Arc<RwLock<Option<NatsClient>>>;
type NatsUrl = Arc<String>;

#[derive(Parser, Debug, Clone)]
struct Opts {
    #[arg(short, long, default_value_t = 1)]
    number: u16,
    #[arg(long, default_value = "subgraph")]
    profile: String,
    #[arg(long, env = "NATS_URL", default_value = "nats://127.0.0.1:4222")]
    nats_url: String,
    #[arg(long, default_value_t = false)]
    listen_any: bool,
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

async fn graphql_handler(State(schema): State<Schema>, req: GraphQLRequest) -> GraphQLResponse {
    schema.execute(req.into_inner()).await.into()
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

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let opts = Opts::parse();

    let id_str = opts.number.to_string();
    let title = format!("{}-{}", opts.profile, id_str);
    let port = 9000 + opts.number as u16;

    let storage: Storage = Arc::new(RwLock::new(0));
    let (tx, _) = broadcast::channel::<i32>(100);
    let sender: Sender = Arc::new(tx);

    let nats_pool: NatsPool = Arc::new(RwLock::new(None));
    let nats_url: NatsUrl = Arc::new(opts.nats_url.clone());

    let query_name = "Query";
    let mutation_name = "Mutation";
    let subscription_name = "Subscription";

    let q_field = format!("{}QueryValue", opts.profile);
    let m_field = format!("{}IncrementValue", opts.profile);
    let s_field = format!("{}OnValueChange", opts.profile);

    let subject = format!("gema.{}.value.updated", opts.profile);

    let mut query = Object::new(query_name);
    let mut mutation = Object::new(mutation_name);
    let mut subscription = Subscription::new(subscription_name);

    let storage_q = storage.clone();
    query = query.field({
        let f = Field::new(q_field.clone(), TypeRef::named_nn(TypeRef::INT), move |_ctx| {
            let storage = storage_q.clone();
            FieldFuture::new(async move {
                let v = *storage.read().await;
                Ok(Some(FieldValue::value(v)))
            })
        });
        f
    });

    let storage_m = storage.clone();
    let sender_m = sender.clone();
    let nats_pool_m = nats_pool.clone();
    let nats_url_m = nats_url.clone();
    mutation = mutation.field({
        let mut f = Field::new(m_field.clone(), TypeRef::named_nn(TypeRef::INT), move |ctx| {
            let storage = storage_m.clone();
            let sender = sender_m.clone();
            let nats_pool = nats_pool_m.clone();
            let nats_url = nats_url_m.clone();
            let subject = subject.clone();
            FieldFuture::new(async move {
                let by = ctx
                    .args
                    .try_get("by")
                    .ok()
                    .and_then(|a| a.i64().ok())
                    .map(|v| v as i32)
                    .unwrap_or(1);
                let mut w = storage.write().await;
                *w += by;
                let new_val = *w;
                let _ = sender.send(new_val);
                let payload = format!(r#"{{"value":{}}}"#, new_val).into_bytes();
                nats_publish(nats_pool, nats_url, subject.clone(), payload).await;
                Ok(Some(FieldValue::value(new_val)))
            })
        });
        f = f.argument(InputValue::new("by", TypeRef::named(TypeRef::INT)));
        f
    });

    let sender_s = sender.clone();
    subscription = subscription.field({
        let sender_inner = sender_s.clone();
        SubscriptionField::new(s_field.clone(), TypeRef::named_nn(TypeRef::INT), move |_ctx| {
            let sender_clone = sender_inner.clone();
            SubscriptionFieldFuture::new(async move {
                let mut rx = sender_clone.subscribe();
                let stream = async_stream::stream! {
                    while let Ok(v) = rx.recv().await {
                        yield Ok(FieldValue::value(v));
                    }
                };
                Ok(stream)
            })
        })
    });

    let schema = Schema::build(query_name, Some(mutation_name), Some(subscription_name))
        .enable_federation()
        .data(storage.clone())
        .data(sender.clone())
        .data(nats_pool.clone())
        .data(nats_url.clone())
        .register(query)
        .register(mutation)
        .register(subscription)
        .finish()?;

    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST])
        .allow_headers(Any)
        .allow_origin(Any);

    let app = Router::new()
        .route(
            "/",
            get({
                let t = title.clone();
                move || graphiql(t.clone())
            }),
        )
        .route("/graphql", post(graphql_handler))
        .route("/graphql", get_service(GraphQLSubscription::new(schema.clone())))
        .layer(cors)
        .with_state(schema);

    let bind_addr = if opts.listen_any { "0.0.0.0" } else { "127.0.0.1" };
    let listener = tokio::net::TcpListener::bind((bind_addr, port)).await?;
    println!("http://{}:{}/", bind_addr, port);
    println!("/graphql");
    axum::serve(listener, app).await?;
    Ok(())
}
