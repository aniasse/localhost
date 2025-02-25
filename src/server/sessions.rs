use crate::server::content_type;
use crate::server::config::ServerConfig;
use crate::type_aliases::Bytes;
use http::header::{CONTENT_LENGTH, CONTENT_TYPE, COOKIE, HOST, SET_COOKIE};
use http::response::Builder;
use http::{HeaderValue, Request, Response, StatusCode};
use std::fs;

type Cookie = str;

// Fonction pour mettre à jour un cookie
pub fn update_cookie(
    req: &Request<Bytes>,
    conf: &ServerConfig,
) -> Result<Response<Bytes>, StatusCode> {
    // Vérifier si le cookie existe déjà
    if req.headers().iter().any(|(_, v)| {
        v.to_str()
            .unwrap_or_default()
            .to_ascii_lowercase()
            .eq("session=cookie")
    }) {
        // Supprimer le cookie s'il existe
        return remove_cookie(
            Response::builder()
                .status(StatusCode::OK)
                .version(req.version()),
            "session=cookie",
        )
        .header(HOST, conf.host)
        .body(vec![])
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR);
    }

    // Définir un nouveau cookie
    set_cookie(
        Response::builder()
            .status(StatusCode::OK)
            .version(req.version()),
        "session=cookie",
    )
    .header(HOST, conf.host)
    .body(vec![])
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

// Fonction pour valider un cookie
pub fn validate_cookie(
    req: &Request<Bytes>,
    conf: &ServerConfig,
) -> Result<Response<Bytes>, StatusCode> {
    // Récupérer la valeur du cookie
    let value = get_cookie(req, "session=cookie")
        .ok_or(StatusCode::UNAUTHORIZED)?
        .to_str()
        .unwrap_or_default();

    // Répondre avec la valeur du cookie
    cookie(
        Response::builder()
            .status(StatusCode::OK)
            .version(req.version()),
        value,
    )
    .header(HOST, conf.host)
    .body(vec![])
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

// Fonction pour démontrer l'utilisation des cookies
pub fn cookie_demo(
    req: &Request<Bytes>,
    config: &ServerConfig,
) -> Result<Response<Bytes>, StatusCode> {
    // Lire le contenu du fichier de démonstration des cookies
    let body = fs::read("./assets/cookie-demo.html").map_err(|_| StatusCode::NOT_FOUND)?;
    let mut resp = Response::builder()
        .version(req.version())
        .header(HOST, config.host)
        .status(StatusCode::OK)
        .header(CONTENT_TYPE, content_type("./assets/cookie_demo.html"))
        .header(CONTENT_LENGTH, body.len());

    // Ajouter les en-têtes standards
    for (key, value) in req.headers() {
        if crate::server::methods::safe::STANDARD_HEADERS.contains(key) {
            resp = resp.header(key, value);
        }
    }

    resp.body(body)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

// Fonction pour définir un cookie dans la réponse
pub fn set_cookie(resp: Builder, value: &Cookie) -> Builder {
    let value = format!("{value}; path=/; Max-Age=3600"); // Expire dans 1 heure
    resp.header(SET_COOKIE, value)
}

// Fonction pour supprimer un cookie de la réponse
pub fn remove_cookie(resp: Builder, value: &Cookie) -> Builder {
    let value = format!("{value}; path=/; expires=Thu, 01 Jan 1970 00:00:00 GMT");
    resp.header(SET_COOKIE, value)
}

/// # cookie
///
/// Ajoute un cookie aux en-têtes de la réponse. Le cookie est spécifié par `value`.
pub fn cookie(resp: Builder, value: &Cookie) -> Builder {
    resp.header(COOKIE, value)
}

// Fonction pour récupérer un cookie à partir de la requête
pub fn get_cookie<'a>(req: &'a Request<Bytes>, value: &'a Cookie) -> Option<&'a HeaderValue> {
    req.headers()
        .get_all(COOKIE)
        .iter()
        .find(|&c| c.to_str().unwrap_or_default().eq(value))
}
