use std::{
    io::{self, BufRead, Write},
    time::Duration,
};

use anyhow::{anyhow, bail, Result};
use reqwest::Client;
use serde_json::{json, Value};

use crate::{
    cli,
    config::local_base,
    version::{product_name, QORX_VERSION},
};

const PROTOCOL_VERSION: &str = "2024-11-05";

#[derive(Debug, Clone, Copy)]
enum Framing {
    Line,
    ContentLength,
}

pub async fn run_stdio() -> Result<()> {
    let client = Client::new();
    let stdin = io::stdin();
    let mut input = stdin.lock();
    let mut stdout = io::stdout();

    while let Some((request, framing)) = read_message(&mut input)? {
        let Some(method) = request.get("method").and_then(Value::as_str) else {
            let id = request.get("id").cloned().unwrap_or(Value::Null);
            write_response(
                &mut stdout,
                framing,
                error_response(id, -32600, "missing method"),
            )?;
            continue;
        };

        let id = request.get("id").cloned();
        if id.is_none() {
            if method == "exit" {
                break;
            }
            continue;
        }
        let id = id.unwrap_or(Value::Null);

        let result = match method {
            "initialize" => Ok(initialize_result()),
            "ping" => Ok(json!({})),
            "tools/list" => Ok(json!({ "tools": tool_definitions() })),
            "tools/call" => {
                call_tool(&client, request.get("params").cloned().unwrap_or_default()).await
            }
            "resources/list" => Ok(json!({ "resources": [] })),
            "prompts/list" => Ok(json!({ "prompts": [] })),
            "shutdown" => Ok(Value::Null),
            _ => Err(anyhow!("unknown MCP method: {method}")),
        };

        match result {
            Ok(result) => write_response(&mut stdout, framing, success_response(id, result))?,
            Err(err) => write_response(
                &mut stdout,
                framing,
                error_response(id, -32603, &err.to_string()),
            )?,
        }
    }

    Ok(())
}

fn read_message<R: BufRead>(input: &mut R) -> Result<Option<(Value, Framing)>> {
    loop {
        let mut line = String::new();
        let read = input.read_line(&mut line)?;
        if read == 0 {
            return Ok(None);
        }

        let line = line.trim_end_matches(['\r', '\n']);
        if line.is_empty() {
            continue;
        }

        if let Some(raw_length) = line
            .to_ascii_lowercase()
            .strip_prefix("content-length:")
            .map(str::trim)
            .map(str::to_string)
        {
            let mut length = raw_length.parse::<usize>()?;
            loop {
                let mut header = String::new();
                input.read_line(&mut header)?;
                let header = header.trim_end_matches(['\r', '\n']);
                if header.is_empty() {
                    break;
                }
                if let Some(raw_length) = header
                    .to_ascii_lowercase()
                    .strip_prefix("content-length:")
                    .map(str::trim)
                    .map(str::to_string)
                {
                    length = raw_length.parse::<usize>()?;
                }
            }

            let mut body = vec![0; length];
            input.read_exact(&mut body)?;
            return Ok(Some((
                serde_json::from_slice::<Value>(&body)?,
                Framing::ContentLength,
            )));
        }

        return Ok(Some((serde_json::from_str::<Value>(line)?, Framing::Line)));
    }
}

fn initialize_result() -> Value {
    json!({
        "protocolVersion": PROTOCOL_VERSION,
        "capabilities": {
            "tools": {}
        },
        "serverInfo": {
            "name": "qorx",
            "version": QORX_VERSION
        },
        "instructions": format!("Use {} v{} tools for local context, session handles, reduction reports, and evidence packs. When the user asks about the local repo, workspace, metrics, or evidence-backed context, call qorx.context_inject first and then pull narrower proof pages only if needed. Do not paste bulk repository or vault contents when a qorx:// handle or Qorx tool can resolve it locally.", product_name(), QORX_VERSION)
    })
}

fn tool_definitions() -> Vec<Value> {
    vec![
        tool(
            "qorx.health",
            "Check whether the local Qorx gateway is reachable.",
            empty_schema(),
        ),
        tool(
            "qorx.stats",
            "Return Qorx reduction, cache, context, and ledger counters.",
            empty_schema(),
        ),
        tool(
            "qorx.session",
            "Return the current qorx:// session pointer for the indexed local context.",
            empty_schema(),
        ),
        tool(
            "qorx.squeeze",
            "Extract a compact evidence pack for a query from local indexed context.",
            json!({
                "type": "object",
                "properties": {
                    "query": {"type": "string", "description": "Question or objective to resolve locally."},
                    "budget_tokens": {"type": "integer", "description": "Maximum model-visible token budget."},
                    "limit": {"type": "integer", "description": "Maximum number of local evidence items."}
                },
                "required": ["query"]
            }),
        ),
        tool(
            "qorx.map",
            "Map a query or diff to related local files, symbols, and context edges.",
            json!({
                "type": "object",
                "properties": {
                    "query": {"type": "string", "description": "Question, feature, bug, or review objective."},
                    "budget_tokens": {"type": "integer", "description": "Maximum model-visible token budget."},
                    "diff": {"type": "string", "description": "Optional unified diff to analyze."}
                },
                "required": ["query"]
            }),
        ),
        tool(
            "qorx.orcl",
            "Return Qorx ORCL ranked contracts, bounded links, and exact quark context for a query or diff.",
            json!({
                "type": "object",
                "properties": {
                    "query": {"type": "string", "description": "Question, feature, bug, or review objective."},
                    "budget_tokens": {"type": "integer", "description": "Maximum model-visible token budget."},
                    "depth": {"type": "integer", "description": "Maximum local link traversal depth."},
                    "limit": {"type": "integer", "description": "Maximum number of ranked contracts."},
                    "diff": {"type": "string", "description": "Optional unified diff to analyze."}
                },
                "required": ["query"]
            }),
        ),
        tool(
            "qorx.strict_answer",
            "Answer from local indexed evidence only, with a small evidence limit.",
            json!({
                "type": "object",
                "properties": {
                    "question": {"type": "string", "description": "Question to answer from local evidence."},
                    "limit": {"type": "integer", "description": "Maximum number of evidence items."}
                },
                "required": ["question"]
            }),
        ),
        tool(
            "qorx.ground",
            "Run the Qorx Grounding Gate: strict evidence, squeeze, B2C planning, optional answer judging, and what-if savings math.",
            json!({
                "type": "object",
                "properties": {
                    "query": {"type": "string", "description": "Question or claim area to ground against local evidence."},
                    "answer": {"type": "string", "description": "Optional answer text to judge claim-by-claim."},
                    "budget_tokens": {"type": "integer", "description": "Maximum model-visible token budget."},
                    "limit": {"type": "integer", "description": "Maximum number of local evidence items."},
                    "raw_tokens": {"type": "integer", "description": "Optional what-if raw input token count."},
                    "sent_tokens": {"type": "integer", "description": "Optional what-if sent input token count."},
                    "input_usd_per_million": {"type": "number", "description": "Optional what-if input price per million tokens."}
                },
                "required": ["query"]
            }),
        ),
        tool(
            "qorx.context_inject",
            "First-call Qorx context pointer for the current agent turn. Use this when local repo, workspace, metrics, or evidence-backed context matters; skip it for ordinary chat.",
            json!({
                "type": "object",
                "properties": {
                    "objective": {"type": "string", "description": "Current user objective or task."},
                    "cwd": {"type": "string", "description": "Optional caller working directory for context-root diagnostics."},
                    "budget_tokens": {"type": "integer", "description": "Maximum model-visible token budget."},
                    "limit": {"type": "integer", "description": "Maximum number of evidence items."}
                }
            }),
        ),
    ]
}

fn tool(name: &str, description: &str, input_schema: Value) -> Value {
    json!({
        "name": name,
        "description": description,
        "inputSchema": input_schema
    })
}

fn empty_schema() -> Value {
    json!({
        "type": "object",
        "properties": {}
    })
}

async fn call_tool(client: &Client, params: Value) -> Result<Value> {
    let name = params
        .get("name")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("tools/call missing tool name"))?;
    let args = params
        .get("arguments")
        .cloned()
        .unwrap_or_else(|| json!({}));

    let output = match name {
        "qorx.health" => gateway_get(client, "/health").await?,
        "qorx.stats" => gateway_get(client, "/stats").await?,
        "qorx.session" => gateway_get(client, "/session").await?,
        "qorx.squeeze" => {
            let query = required_string(&args, "query")?;
            gateway_post(
                client,
                "/squeeze",
                json!({
                    "query": query,
                    "budget_tokens": optional_u64(&args, "budget_tokens"),
                    "limit": optional_u64(&args, "limit")
                }),
            )
            .await?
        }
        "qorx.map" => {
            let query = required_string(&args, "query")?;
            gateway_post(
                client,
                "/map",
                json!({
                    "query": query,
                    "budget_tokens": optional_u64(&args, "budget_tokens"),
                    "diff": args.get("diff").and_then(Value::as_str)
                }),
            )
            .await?
        }
        "qorx.orcl" => {
            let query = required_string(&args, "query")?;
            gateway_post(
                client,
                "/orcl",
                json!({
                    "query": query,
                    "budget_tokens": optional_u64(&args, "budget_tokens"),
                    "depth": optional_u64(&args, "depth"),
                    "limit": optional_u64(&args, "limit"),
                    "diff": args.get("diff").and_then(Value::as_str)
                }),
            )
            .await?
        }
        "qorx.strict_answer" => {
            let question = required_string(&args, "question")?;
            gateway_post(
                client,
                "/strict-answer",
                json!({
                    "question": question,
                    "limit": optional_u64(&args, "limit")
                }),
            )
            .await?
        }
        "qorx.ground" => {
            let query = required_string(&args, "query")?;
            gateway_post(
                client,
                "/ground",
                json!({
                    "query": query,
                    "answer": args.get("answer").and_then(Value::as_str),
                    "budget_tokens": optional_u64(&args, "budget_tokens"),
                    "limit": optional_u64(&args, "limit"),
                    "raw_tokens": optional_u64(&args, "raw_tokens"),
                    "sent_tokens": optional_u64(&args, "sent_tokens"),
                    "input_usd_per_million": args.get("input_usd_per_million").and_then(Value::as_f64)
                }),
            )
            .await?
        }
        "qorx.context_inject" => {
            gateway_post(
                client,
                "/context/inject",
                json!({
                    "objective": args.get("objective").and_then(Value::as_str),
                    "cwd": args.get("cwd").and_then(Value::as_str).map(str::to_string).or_else(|| std::env::current_dir().ok().map(|path| path.display().to_string())),
                    "budget_tokens": optional_u64(&args, "budget_tokens"),
                    "limit": optional_u64(&args, "limit")
                }),
            )
            .await?
        }
        _ => bail!("unknown Qorx tool: {name}"),
    };

    Ok(json!({
        "content": [
            {
                "type": "text",
                "text": serde_json::to_string_pretty(&output)?
            }
        ]
    }))
}

async fn gateway_get(client: &Client, path: &str) -> Result<Value> {
    cli::ensure_daemon().await?;
    let response = client
        .get(format!("{}{}", local_base(), path))
        .timeout(Duration::from_secs(10))
        .send()
        .await?;
    response_json(response).await
}

async fn gateway_post(client: &Client, path: &str, body: Value) -> Result<Value> {
    cli::ensure_daemon().await?;
    let response = client
        .post(format!("{}{}", local_base(), path))
        .json(&body)
        .timeout(Duration::from_secs(20))
        .send()
        .await?;
    response_json(response).await
}

async fn response_json(response: reqwest::Response) -> Result<Value> {
    let status = response.status();
    let text = response.text().await?;
    if !status.is_success() {
        bail!("Qorx gateway returned {status}: {text}");
    }
    Ok(serde_json::from_str(&text).unwrap_or_else(|_| json!({ "text": text })))
}

fn required_string<'a>(args: &'a Value, key: &str) -> Result<&'a str> {
    args.get(key)
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| anyhow!("missing required string argument: {key}"))
}

fn optional_u64(args: &Value, key: &str) -> Option<u64> {
    args.get(key).and_then(Value::as_u64)
}

fn success_response(id: Value, result: Value) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": result
    })
}

fn error_response(id: Value, code: i64, message: &str) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": {
            "code": code,
            "message": message
        }
    })
}

fn write_response(stdout: &mut io::Stdout, framing: Framing, value: Value) -> Result<()> {
    let body = serde_json::to_string(&value)?;
    match framing {
        Framing::Line => {
            writeln!(stdout, "{body}")?;
        }
        Framing::ContentLength => {
            write!(stdout, "Content-Length: {}\r\n\r\n{body}", body.len())?;
        }
    }
    stdout.flush()?;
    Ok(())
}
