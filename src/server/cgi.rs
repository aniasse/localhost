use crate::log;
use crate::log::*;
use crate::server::path::add_root_to_path;
use crate::server::{get_route, Bytes, ServerConfig, StatusCode};
use crate::type_aliases::FileExtension;
use http::header::*;
use http::{HeaderMap, HeaderName, HeaderValue, Request, Response};
use std::env;
use std::process::Command;

// Enumération pour définir les types de scripts CGI supportés
#[derive(Clone, Debug)]
pub enum Cgi {
    PHP,
    Python,
}

// Fonction pour vérifier si une requête est destinée à un script CGI
pub fn is_cgi_request(path: &str) -> bool {
    path.contains("/cgi/")
}

// En-têtes standards à inclure dans la réponse
const STANDARD_HEADERS: [HeaderName; 1] = [TRANSFER_ENCODING];

// Fonction principale pour exécuter un script CGI
pub fn execute_cgi_script(
    req: &Request<Bytes>,
    config: &ServerConfig,
) -> Result<Response<Bytes>, StatusCode> {
    // Récupérer la route correspondant à la requête
    let route = match get_route(req, config) {
        Ok(route) => route,
        Err((status, _)) => return Err(status),
    };

    // Vérifier les paramètres de la route
    let settings = match &route.settings {
        Some(s) => s,
        None => return Err(StatusCode::BAD_REQUEST),
    };

    // Vérifier si un script CGI est défini pour cette route
    if settings.cgi_def.is_none() {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Construire le chemin complet du script CGI
    let full_path = add_root_to_path(&route, req.uri().path());
    let body = match String::from_utf8(req.body().clone()) {
        Ok(b) => b,
        Err(_) => return Err(StatusCode::BAD_REQUEST),
    };

    // Extraire l'extension du fichier pour déterminer le type de script CGI
    let extension = full_path.split('.').rev().collect::<Vec<&str>>()[0].trim_end();

    let mut file_extension = String::new();

    for ch in extension.chars() {
        if ch.is_alphanumeric() {
            file_extension.push(ch);
        } else {
            break;
        }
    }

    let path = full_path
        .split(&format!(".{file_extension}"))
        .collect::<Vec<&str>>()[0]
        .to_string();

    let path = format!("{path}.{file_extension}");
    add_env_variables(req, config, file_extension.as_str());

    // Vérifier si l'extension du fichier est associée à un script CGI
    let (command, arguments) = match settings
        .cgi_def
        .clone()
        .unwrap()
        .get(file_extension.as_str())
    {
        Some(cgi_type) => match cgi_type {
            Cgi::PHP => ("php", vec![path, body]),
            Cgi::Python => ("python3", vec![path, body]),
        },

        None => {
            log!(
                LogFileType::Server,
                format!("Error: CGI not found {}", path)
            );
            return Err(StatusCode::NOT_FOUND);
        }
    };

    // Exécuter le script CGI et capturer sa sortie
    let body = match Command::new(command).args(arguments).output() {
        Ok(output) => output.stdout,
        Err(e) => {
            log!(
                LogFileType::Server,
                format!("Error executing CGI script: {}", e)
            );
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Construire la réponse HTTP
    let mut resp = Response::builder()
        .version(req.version())
        .header(HOST, config.host)
        .status(StatusCode::OK)
        .header(CONTENT_TYPE, "text/html")
        .header(CONTENT_LENGTH, body.len());

    // Ajouter les en-têtes standards à la réponse
    for (key, value) in req.headers() {
        if STANDARD_HEADERS.contains(key) {
            resp = resp.header(key, value);
        }
    }

    let response = resp
        .body(body)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(response)
}

// Fonction pour ajouter des variables d'environnement nécessaires au script CGI
fn add_env_variables(req: &Request<Bytes>, config: &ServerConfig, file_extension: FileExtension) {
    add_http_variables(req.headers());
    if let Some(query) = req.uri().query() {
        env::set_var("QUERY_STRING", query);
    }

    env::set_var("REQUEST_METHOD", req.method().to_string());
    env::set_var("SERVER_NAME", config.host);

    if let Some(port) = req.uri().port_u16() {
        env::set_var("SERVER_PORT", format!("{port}"));
    }

    env::set_var("SERVER_SOFTWARE", "Rust v1.74.0");

    let path = req
        .uri()
        .path()
        .split(file_extension)
        .collect::<Vec<&str>>();

    // Exemple : localhost:8080/cgi/python.py/path/to/file -> PATH_INFO: /path/to/file
    if contains_path_info(path.clone()) {
        env::set_var("PATH_INFO", path[1]);
    }
}

// Fonction pour ajouter les variables d'environnement HTTP
fn add_http_variables(headers: &HeaderMap<HeaderValue>) {
    for (key, v) in headers {
        let value = v.to_str().unwrap_or_default();
        if value.is_empty() {
            continue;
        }
        match *key {
            ACCEPT => env::set_var("HTTP_ACCEPT", value),
            CONTENT_LENGTH => env::set_var("CONTENT_LENGTH", value),
            CONTENT_TYPE => env::set_var("CONTENT_TYPE", value),
            ACCEPT_CHARSET => env::set_var("HTTP_ACCEPT_CHARSET", value),
            ACCEPT_ENCODING => env::set_var("HTTP_ACCEPT_ENCODING", value),
            ACCEPT_LANGUAGE => env::set_var("HTTP_ACCEPT_LANGUAGE", value),
            FORWARDED => env::set_var("HTTP_FORWARDED", value),
            HOST => env::set_var("HTTP_HOST", value),
            PROXY_AUTHORIZATION => env::set_var("HTTP_PROXY_AUTHORIZATION", value),
            USER_AGENT => env::set_var("HTTP_USER_AGENT", value),
            COOKIE => env::set_var("COOKIE", value),
            _ => {}
        }
    }
}

// Fonction pour vérifier si le chemin contient des informations de chemin supplémentaires
fn contains_path_info(path: Vec<&str>) -> bool {
    path.len() == 2
}
