use ic_cdk::api::management_canister::http_request::{HttpResponse, TransformArgs};
use ic_cdk_macros::*;

const BACKEND_CANISTER_ID: &str = "bkyz2-fmaaa-aaaaa-qaaaq-cai";

#[update]
fn greet(name: String) -> String {
    format!("Hello, {}!", name)
}

#[query]
fn get_html() -> String {
    format!(r#"
    <!DOCTYPE html>
    <html>
    <head>
        <title>HTTP Request Demo</title>
    </head>
    <body>
        <h1>HTTP Request Demo</h1>
        <button onclick="sendRequest()">Send HTTP Request</button>
        <div id="result"></div>

        <script>
            async function sendRequest() {{
                try {{
                    const agent = new window.ic.HttpAgent();
                    const backend = window.ic.canister('{backend_id}');
                    const result = await backend.make_http_request();
                    document.getElementById('result').innerText = result;
                }} catch (e) {{
                    document.getElementById('result').innerText = 'Error: ' + e.message;
                }}
            }}
        </script>
        <script src="https://cdn.jsdelivr.net/npm/@dfinity/agent/lib/index.js"></script>
    </body>
    </html>
    "#, backend_id = BACKEND_CANISTER_ID)
} 