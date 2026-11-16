use crate::communicator::{json_escape, ToolCommunicator, ToolParams};
use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;

const EMBEDDED_INDEX_HTML: &str = include_str!("../web/index.html");
const EMBEDDED_STYLES_CSS: &str = include_str!("../web/styles.css");
const EMBEDDED_APP_JS: &str = include_str!("../web/app.js");

const EMBEDDED_PUB_INDEX_HTML: &str = include_str!("../web_publico/index.html");
const EMBEDDED_PUB_STYLES_CSS: &str = include_str!("../web_publico/styles.css");
const EMBEDDED_PUB_APP_JS: &str = include_str!("../web_publico/app.js");

pub struct HttpResponse {
    pub status_code: u16,
    pub status_text: &'static str,
    pub content_type: String,
    pub extra_headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

impl HttpResponse {
    pub fn ok_json(json: String) -> Self {
        Self {
            status_code: 200,
            status_text: "OK",
            content_type: "application/json; charset=utf-8".to_string(),
            extra_headers: vec![("Cache-Control".to_string(), "no-store".to_string())],
            body: json.into_bytes(),
        }
    }

    pub fn ok_text(content_type: &str, text: String) -> Self {
        Self {
            status_code: 200,
            status_text: "OK",
            content_type: content_type.to_string(),
            extra_headers: vec![("Cache-Control".to_string(), "no-cache".to_string())],
            body: text.into_bytes(),
        }
    }

    pub fn ok_pdf(bytes: Vec<u8>, filename: &str, as_attachment: bool) -> Self {
        let disposition = if as_attachment {
            format!("attachment; filename=\"{}\"", filename)
        } else {
            format!("inline; filename=\"{}\"", filename)
        };
        Self {
            status_code: 200,
            status_text: "OK",
            content_type: "application/pdf".to_string(),
            extra_headers: vec![
                ("Content-Disposition".to_string(), disposition),
                ("Cache-Control".to_string(), "no-store".to_string()),
            ],
            body: bytes,
        }
    }

    pub fn error_json(status_code: u16, status_text: &'static str, message: &str) -> Self {
        let body = format!(
            "{{\"status\":\"error\",\"message\":\"{}\"}}",
            json_escape(message)
        );
        Self {
            status_code,
            status_text,
            content_type: "application/json; charset=utf-8".to_string(),
            extra_headers: Vec::new(),
            body: body.into_bytes(),
        }
    }

    pub fn into_http_bytes(self) -> Vec<u8> {
        let mut header = format!(
            "HTTP/1.1 {} {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n",
            self.status_code,
            self.status_text,
            self.content_type,
            self.body.len()
        );
        for (k, v) in self.extra_headers {
            header.push_str(&format!("{}: {}\r\n", k, v));
        }
        header.push_str("\r\n");

        let mut out = header.into_bytes();
        out.extend_from_slice(&self.body);
        out
    }
}

/// Inicia el servidor HTTP para el Catálogo y Estudio de Materiales Didácticos.
pub fn run_web_server(port: u16, workspace_dir: PathBuf) -> std::io::Result<()> {
    let mut active_port = port;
    let listener = match TcpListener::bind(format!("127.0.0.1:{}", active_port)) {
        Ok(l) => l,
        Err(e) if e.kind() == std::io::ErrorKind::AddrInUse && port == 8080 => {
            active_port = 8787;
            TcpListener::bind(format!("127.0.0.1:{}", active_port))?
        }
        Err(e) => return Err(e),
    };
    let addr = format!("127.0.0.1:{}", active_port);
    let comm = Arc::new(Mutex::new(ToolCommunicator::new(workspace_dir.clone())));

    // Levantar también un puerto independiente dedicado al Portal Público General (active_port + 1)
    let public_port = active_port.saturating_add(1);
    let public_listener_opt = TcpListener::bind(format!("127.0.0.1:{}", public_port)).ok();

    println!("==================================================================");
    println!("  AULAFORJA & RINCÓN DIDÁCTICO: Servidores Web Activos");
    println!("==================================================================");
    println!("  • Estudio Técnico / Comunicador: http://{}", addr);
    println!("  • Portal Público (Ruta):         http://{}/publico", addr);
    if public_listener_opt.is_some() {
        println!(
            "  • Portal Público (Puerto propio): http://127.0.0.1:{}/",
            public_port
        );
    }
    println!("  • Directorio Base:               {}", workspace_dir.display());
    if let Ok(guard) = comm.lock() {
        println!(
            "  • Herramientas en Comunicador:   {} generadores ({} paquetes públicos)",
            guard.list_manifests().len(),
            guard.list_public_presets().len()
        );
    }
    println!("  • Presiona Ctrl+C para detener el servidor.");
    println!();

    if let Some(pub_listener) = public_listener_opt {
        let comm_pub = Arc::clone(&comm);
        let ws_pub = workspace_dir.clone();
        thread::spawn(move || {
            for stream_res in pub_listener.incoming() {
                if let Ok(stream) = stream_res {
                    let c = Arc::clone(&comm_pub);
                    let w = ws_pub.clone();
                    thread::spawn(move || {
                        let _ = handle_client(stream, c, w, true);
                    });
                }
            }
        });
    }

    for stream_res in listener.incoming() {
        match stream_res {
            Ok(stream) => {
                let comm_clone = Arc::clone(&comm);
                let ws_clone = workspace_dir.clone();
                thread::spawn(move || {
                    let _ = handle_client(stream, comm_clone, ws_clone, false);
                });
            }
            Err(e) => {
                eprintln!("Error en conexión entrante: {}", e);
            }
        }
    }

    Ok(())
}

fn handle_client(
    mut stream: TcpStream,
    comm: Arc<Mutex<ToolCommunicator>>,
    workspace_dir: PathBuf,
    public_default_root: bool,
) -> std::io::Result<()> {
    let mut buffer = [0u8; 16384];
    let bytes_read = stream.read(&mut buffer)?;
    if bytes_read == 0 {
        return Ok(());
    }

    let raw_req = String::from_utf8_lossy(&buffer[..bytes_read]);
    let first_line = raw_req.lines().next().unwrap_or("");
    let mut parts = first_line.split_whitespace();
    let method = parts.next().unwrap_or("GET");
    let raw_target = parts.next().unwrap_or("/");

    let effective_target = if public_default_root && (raw_target == "/" || raw_target == "/index.html") {
        "/publico"
    } else {
        raw_target
    };

    let response = route_request(method, effective_target, &comm, &workspace_dir);
    let bytes = response.into_http_bytes();
    stream.write_all(&bytes)?;
    stream.flush()?;
    Ok(())
}

pub fn route_request(
    method: &str,
    raw_target: &str,
    comm: &Arc<Mutex<ToolCommunicator>>,
    workspace_dir: &PathBuf,
) -> HttpResponse {
    let (raw_path, raw_query) = match raw_target.split_once('?') {
        Some((p, q)) => (p, q),
        None => (raw_target, ""),
    };

    let path = percent_decode(raw_path);
    let params = parse_query_params(raw_query);

    // 1. Activos estáticos de la interfaz web (Estudio Técnico y Portal Público)
    if method == "GET" && (path == "/" || path == "/index.html") {
        let html = fs::read_to_string(workspace_dir.join("web/index.html"))
            .unwrap_or_else(|_| EMBEDDED_INDEX_HTML.to_string());
        return HttpResponse::ok_text("text/html; charset=utf-8", html);
    }
    if method == "GET" && path == "/styles.css" {
        let css = fs::read_to_string(workspace_dir.join("web/styles.css"))
            .unwrap_or_else(|_| EMBEDDED_STYLES_CSS.to_string());
        return HttpResponse::ok_text("text/css; charset=utf-8", css);
    }
    if method == "GET" && path == "/app.js" {
        let js = fs::read_to_string(workspace_dir.join("web/app.js"))
            .unwrap_or_else(|_| EMBEDDED_APP_JS.to_string());
        return HttpResponse::ok_text("application/javascript; charset=utf-8", js);
    }

    if method == "GET"
        && (path == "/publico"
            || path == "/publico/"
            || path == "/publico/index.html"
            || path == "/trans"
            || path == "/trans/"
            || path == "/publico/trans"
            || path == "/publico/trans/")
    {
        let html = fs::read_to_string(workspace_dir.join("web_publico/index.html"))
            .unwrap_or_else(|_| EMBEDDED_PUB_INDEX_HTML.to_string());
        return HttpResponse::ok_text("text/html; charset=utf-8", html);
    }
    if method == "GET" && path == "/publico/styles.css" {
        let css = fs::read_to_string(workspace_dir.join("web_publico/styles.css"))
            .unwrap_or_else(|_| EMBEDDED_PUB_STYLES_CSS.to_string());
        return HttpResponse::ok_text("text/css; charset=utf-8", css);
    }
    if method == "GET" && path == "/publico/app.js" {
        let js = fs::read_to_string(workspace_dir.join("web_publico/app.js"))
            .unwrap_or_else(|_| EMBEDDED_PUB_APP_JS.to_string());
        return HttpResponse::ok_text("application/javascript; charset=utf-8", js);
    }

    // 2. Endpoints del Comunicador Backend
    if method == "GET" && path == "/api/presets" {
        let Ok(guard) = comm.lock() else {
            return HttpResponse::error_json(500, "Internal Server Error", "Lock error");
        };
        let presets = guard.list_public_presets();
        let mut json = String::from("{\"status\":\"ok\",\"presets\":[");
        for (i, p) in presets.iter().enumerate() {
            if i > 0 {
                json.push(',');
            }
            json.push_str(&p.to_json());
        }
        json.push_str("]}");
        return HttpResponse::ok_json(json);
    }
    if method == "GET" && path == "/api/tools" {
        let Ok(guard) = comm.lock() else {
            return HttpResponse::error_json(500, "Internal Server Error", "Lock error");
        };
        let manifests = guard.list_manifests();
        let mut json = format!(
            "{{\"status\":\"ok\",\"lexicon_count\":{},\"tools\":[",
            guard.lexicon_count()
        );
        for (i, m) in manifests.iter().enumerate() {
            if i > 0 {
                json.push(',');
            }
            json.push_str(&m.to_json());
        }
        json.push_str("]}");
        return HttpResponse::ok_json(json);
    }

    if method == "POST" && path == "/api/communicator/rescan" {
        let Ok(mut guard) = comm.lock() else {
            return HttpResponse::error_json(500, "Internal Server Error", "Lock error");
        };
        guard.discover_external_tools();
        let count = guard.list_manifests().len();
        return HttpResponse::ok_json(format!(
            "{{\"status\":\"ok\",\"tools_count\":{}}}",
            count
        ));
    }

    // Rutas /api/tools/<tool_id>/<action>
    if let Some(rest) = path.strip_prefix("/api/tools/") {
        if let Some((tool_id, action)) = rest.split_once('/') {
            let Ok(guard) = comm.lock() else {
                return HttpResponse::error_json(500, "Internal Server Error", "Lock error");
            };

            match (method, action) {
                ("GET", "preview") | ("POST", "preview") => {
                    return match guard.preview(tool_id, &params) {
                        Ok(preview) => HttpResponse::ok_json(preview.to_json()),
                        Err(err) => HttpResponse::error_json(404, "Not Found", &err),
                    };
                }
                ("GET", "pdf") => {
                    let save = params
                        .get_str("save")
                        .map(|v| v == "1" || v == "true")
                        .unwrap_or(false);
                    let download = params
                        .get_str("download")
                        .map(|v| v == "1" || v == "true")
                        .unwrap_or(false);

                    return match guard.generate_pdf(tool_id, &params, save) {
                        Ok(doc) => HttpResponse::ok_pdf(doc.pdf_bytes, &doc.filename, download),
                        Err(err) => HttpResponse::error_json(400, "Bad Request", &err),
                    };
                }
                ("POST", "save") => {
                    return match guard.generate_pdf(tool_id, &params, true) {
                        Ok(doc) => HttpResponse::ok_json(format!(
                            "{{\"status\":\"ok\",\"filename\":\"{}\",\"seed\":{},\"total_pages\":{}}}",
                            json_escape(&doc.filename),
                            doc.seed,
                            doc.total_pages
                        )),
                        Err(err) => HttpResponse::error_json(400, "Bad Request", &err),
                    };
                }
                _ => {}
            }
        }
    }

    // 3. Biblioteca de materiales generados (/api/materials)
    if method == "GET" && path == "/api/materials" {
        let Ok(guard) = comm.lock() else {
            return HttpResponse::error_json(500, "Internal Server Error", "Lock error");
        };
        let list = guard.list_stored_materials();
        let mut json = String::from("{\"status\":\"ok\",\"materials\":[");
        for (i, item) in list.iter().enumerate() {
            if i > 0 {
                json.push(',');
            }
            json.push_str(&item.to_json());
        }
        json.push_str("]}");
        return HttpResponse::ok_json(json);
    }

    if let Some(filename) = path.strip_prefix("/api/materials/") {
        let Ok(guard) = comm.lock() else {
            return HttpResponse::error_json(500, "Internal Server Error", "Lock error");
        };

        if method == "GET" {
            let download = params
                .get_str("download")
                .map(|v| v == "1" || v == "true")
                .unwrap_or(false);
            return match guard.read_stored_material(filename) {
                Ok(bytes) => HttpResponse::ok_pdf(bytes, filename, download),
                Err(err) => HttpResponse::error_json(404, "Not Found", &err),
            };
        } else if method == "DELETE" {
            return match guard.delete_stored_material(filename) {
                Ok(()) => HttpResponse::ok_json("{\"status\":\"ok\"}".to_string()),
                Err(err) => HttpResponse::error_json(400, "Bad Request", &err),
            };
        }
    }

    HttpResponse::error_json(404, "Not Found", "Ruta no encontrada en el servidor")
}

pub fn parse_query_params(query: &str) -> ToolParams {
    let mut params = ToolParams::new();
    for pair in query.split('&') {
        if pair.is_empty() {
            continue;
        }
        let (k, v) = match pair.split_once('=') {
            Some((k, v)) => (percent_decode(k), percent_decode(v)),
            None => (percent_decode(pair), String::new()),
        };
        if !k.is_empty() {
            params.values.insert(k, v);
        }
    }
    params
}

pub fn percent_decode(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut i = 0;

    while i < bytes.len() {
        match bytes[i] {
            b'+' => {
                out.push(b' ');
                i += 1;
            }
            b'%' if i + 2 < bytes.len() => {
                let h1 = hex_val(bytes[i + 1]);
                let h2 = hex_val(bytes[i + 2]);
                if let (Some(a), Some(b)) = (h1, h2) {
                    out.push((a << 4) | b);
                    i += 3;
                } else {
                    out.push(bytes[i]);
                    i += 1;
                }
            }
            b => {
                out.push(b);
                i += 1;
            }
        }
    }

    String::from_utf8_lossy(&out).into_owned()
}

fn hex_val(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_routes_catalog_preview_and_pdf() {
        let tmp_dir = std::env::temp_dir().join("palabrea_server_test");
        let _ = fs::create_dir_all(&tmp_dir);
        let comm = Arc::new(Mutex::new(ToolCommunicator::new(tmp_dir.clone())));

        // 1. Catálogo /api/tools
        let res_tools = route_request("GET", "/api/tools", &comm, &tmp_dir);
        assert_eq!(res_tools.status_code, 200);
        let body_tools = String::from_utf8_lossy(&res_tools.body);
        assert!(body_tools.contains("\"palabrea\""));
        assert!(body_tools.contains("\"taller-lexico\""));

        // 2. Vista Previa /api/tools/palabrea/preview
        let res_prev = route_request(
            "GET",
            "/api/tools/palabrea/preview?cards=12&format=a4&seed=12345",
            &comm,
            &tmp_dir,
        );
        assert_eq!(res_prev.status_code, 200);
        let body_prev = String::from_utf8_lossy(&res_prev.body);
        assert!(body_prev.contains("\"seed\":12345"));
        assert!(body_prev.contains("\"placed_words\":"));

        // 3. Generación PDF /api/tools/palabrea/pdf
        let res_pdf = route_request(
            "GET",
            "/api/tools/palabrea/pdf?cards=12&format=letter&seed=12345&save=1",
            &comm,
            &tmp_dir,
        );
        assert_eq!(res_pdf.status_code, 200);
        assert_eq!(res_pdf.content_type, "application/pdf");
        assert!(res_pdf.body.starts_with(b"%PDF-1.4"));

        // 4. Biblioteca /api/materials
        let res_mat = route_request("GET", "/api/materials", &comm, &tmp_dir);
        assert_eq!(res_mat.status_code, 200);
        let body_mat = String::from_utf8_lossy(&res_mat.body);
        assert!(body_mat.contains("palabrea_"));

        // 5. Portal Público (/publico) y paquetes preconfigurados (/api/presets)
        let res_pub = route_request("GET", "/publico", &comm, &tmp_dir);
        assert_eq!(res_pub.status_code, 200);
        let body_pub = String::from_utf8_lossy(&res_pub.body);
        assert!(body_pub.contains("Rincón Didáctico"));

        let res_presets = route_request("GET", "/api/presets", &comm, &tmp_dir);
        assert_eq!(res_presets.status_code, 200);
        let body_presets = String::from_utf8_lossy(&res_presets.body);
        assert!(body_presets.contains("\"sumas-iniciales\""));
        assert!(body_presets.contains("\"palabrea-familiar-completo\""));
    }
}
