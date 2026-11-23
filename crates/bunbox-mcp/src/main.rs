//! Local single-owner stdio adapter for the reviewed staged profile.
mod jobs;
mod model;
mod venue;

use anyhow::{Context, Result, ensure};
use jobs::{Store, valid_job};
use rmcp::{ErrorData, RoleServer, ServerHandler, ServiceExt, model::*, service::RequestContext};
use serde::Deserialize;
use serde_json::{Value, json};
use std::{env, sync::Arc};

#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct Lookup {
    job_id: String,
}

struct Server {
    store: Store,
}
impl ServerHandler for Server {
    fn get_info(&self) -> ServerConfig {
        ServerConfig::new(ServerCapabilities::builder().enable_tools().build())
    }
    async fn list_tools(
        &self,
        _: Option<PaginatedRequestParams>,
        _: RequestContext<RoleServer>,
    ) -> std::result::Result<ListToolsResult, ErrorData> {
        let get = json!({"type":"object","additionalProperties":false,"required":["jobId"],
            "properties":{"jobId":{"type":"string","pattern":"^[a-f0-9]{64}$"}}});
        let id = json!({"type":"string","minLength":1,"maxLength":256});
        let row = json!({"type":"object","additionalProperties":false,"required":["sourceRef","threadRef","timestamp","direction"],
            "properties":{"sourceRef":id,"threadRef":{"type":["string","null"]},"timestamp":{"type":"integer"},
            "direction":{"enum":["inbound","outbound","auto","draft","unknown"]},"subject":{"type":"string","maxLength":1024}}});
        let input = json!({"type":"object","additionalProperties":false,"required":["rows","cutoff","coverage"],
            "properties":{"rows":{"type":"array","items":row,"maxItems":2000}, "cutoff":{"type":"integer"},
            "coverage":{"type":"object","required":["inbox","sent"],"additionalProperties":false,
            "properties":{"inbox":{"type":"boolean"},"sent":{"type":"boolean"}}}}});
        let submit = json!({"type":"object","additionalProperties":false,"required":["requestKey","recipeSha256","input"],
            "properties":{"requestKey":id,"recipeSha256":{"const":model::digest(model::RECIPE)},"input":input}});
        let descriptions = [
            (
                "submit",
                "Submit the pinned, read-only unanswered-demo recipe",
                submit,
            ),
            (
                "get",
                "Read a job accepted for this owner and run",
                get.clone(),
            ),
            (
                "cancel",
                "Request cancellation and read the current disposition",
                get,
            ),
        ];
        let mut tools = Vec::new();
        for (name, description, schema) in descriptions {
            let object = schema
                .as_object()
                .ok_or_else(|| ErrorData::internal_error("invalid_schema", None))?;
            tools.push(Tool::new(name, description, Arc::new(object.clone())));
        }
        Ok(ListToolsResult::with_all_items(tools))
    }
    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        _: RequestContext<RoleServer>,
    ) -> std::result::Result<CallToolResponse, ErrorData> {
        let args = Value::Object(request.arguments.unwrap_or_default());
        let value = self.invoke(request.name.as_ref(), args).await;
        Ok(match value {
            Ok(v) => CallToolResult::structured(v),
            Err(e) => CallToolResult::structured_error(jobs::failure(&e)),
        }
        .into())
    }
}
impl Server {
    async fn invoke(&self, name: &str, args: Value) -> Result<Value> {
        match name {
            "submit" => self.store.submit(serde_json::from_value(args)?),
            "get" | "cancel" => {
                let lookup: Lookup = serde_json::from_value(args)?;
                ensure!(valid_job(&lookup.job_id), "unknown_job");
                self.store.get(&lookup.job_id, name == "cancel").await
            }
            _ => anyhow::bail!("unknown_tool"),
        }
    }
}
#[tokio::main(flavor = "multi_thread", worker_threads = 2)]
async fn main() -> Result<()> {
    let args: Vec<String> = env::args().skip(1).collect();
    let store = Store::open()?;
    if let [command, id] = args.as_slice() {
        ensure!(command == "--worker" && valid_job(id), "invalid_arguments");
        return store.worker(id).await;
    }
    ensure!(args.is_empty(), "expected_stdio");
    let transport = rmcp::transport::async_rw::AsyncRwTransport::new_server(
        tokio::io::stdin(),
        tokio::io::stdout(),
    )
    .with_max_line_length(model::MAX_STAGED_BYTES + 4096);
    let server = Server { store }
        .serve(transport)
        .await
        .context("stdio_initialization")?;
    server.waiting().await?;
    Ok(())
}
