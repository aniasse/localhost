use config::route::Settings;
use http::StatusCode;
use std::collections::HashMap;

// Importation des modules nécessaires
pub use crate::server::*;

// Fonction pour configurer les paramètres du serveur
pub fn server_config() -> Vec<ServerConfig<'static>> {
    vec![ServerConfig {
        // Adresse IP sur laquelle le serveur écoute.
        // Modifiez "127.0.0.1" par l'adresse IP de votre serveur si nécessaire.
        host: "127.0.0.1",

        // Ports sur lesquels le serveur écoutera. Ajoutez ou supprimez des ports selon vos besoins.
        ports: vec![8080, 8081],

        // Chemin pour les pages d'erreur personnalisées. Définissez sur 'Some(path)' pour activer, ou laissez 'None' pour la gestion des erreurs par défaut.
        custom_error_path: None,

        // Taille maximale autorisée pour les corps de requête en octets. Ajustez selon vos besoins.
        body_size_limit: 1000000000024,

        // Configuration des routes individuelles sur le serveur.
        routes: vec![
            Route {
                // Chemin pour la route. Ajustez-le pour correspondre à l'endpoint que vous souhaitez configurer.
                url_path: "/api/update-cookie",
                // Méthodes HTTP autorisées pour cette route. Ajoutez ou supprimez des méthodes selon vos besoins.
                methods: vec![http::Method::POST],
                // Fonction de gestion pour la route. Changez 'update_cookie' par votre fonction personnalisée si nécessaire.
                handler: Some(update_cookie),
                // Paramètres spécifiques à la route. Laissez 'None' pour les paramètres par défaut.
                settings: None,
            },
            // Routes supplémentaires suivant la même structure. Personnalisez chaque route selon vos besoins.
            Route {
                url_path: "/api/get-cookie",
                methods: vec![http::Method::GET],
                handler: Some(validate_cookie),
                settings: None,
            },
            Route {
                url_path: "/api/cookie-demo",
                methods: vec![http::Method::GET],
                handler: Some(cookie_demo),
                settings: None,
            },
            Route {
                url_path: "/cgi",
                methods: vec![http::Method::GET],
                handler: None, // Pas de gestionnaire spécifique signifie que le traitement est défini par 'settings'.
                settings: Some(Settings {
                    // Configuration pour les scripts CGI.
                    cgi_def: Some(HashMap::from([
                        // Associez les extensions de fichier aux gestionnaires CGI. Ajoutez ou supprimez des mappages selon vos besoins.
                        ("php", Cgi::PHP),
                        ("py", Cgi::Python),
                    ])),
                    // Activez l'affichage du contenu du répertoire pour cette route. Définissez sur 'false' pour désactiver.
                    list_directory: true,
                    // Paramètres CGI supplémentaires peuvent être configurés ici.
                    // Laissez 'None' pour les valeurs par défaut ou spécifiez pour personnaliser le comportement.
                    http_redirections: None,
                    redirect_status_code: None,
                    root_path: None,
                    default_if_url_is_dir: None,
                    default_if_request_is_dir: None,
                }),
            },
            Route {
                url_path: "/test.txt",
                methods: vec![http::Method::GET, http::Method::POST],
                handler: None,
                settings: Some(Settings {
                    http_redirections: Some(vec!["/redirection-test"]),
                    redirect_status_code: Some(StatusCode::from_u16(301).unwrap()),
                    root_path: Some("/assets"),
                    default_if_url_is_dir: None,
                    default_if_request_is_dir: None,
                    cgi_def: None,
                    list_directory: false,
                }),
            },
            Route {
                url_path: "/mega-dir",
                methods: vec![http::Method::GET],
                handler: None,
                settings: Some(Settings {
                    http_redirections: None,
                    redirect_status_code: None,
                    root_path: Some("/assets"),
                    default_if_url_is_dir: Some("/dir.html"),
                    default_if_request_is_dir: None,
                    cgi_def: None,
                    list_directory: false,
                }),
            },
            Route {
                url_path: "/src",
                methods: vec![http::Method::GET],
                handler: None,
                settings: Some(Settings {
                    http_redirections: None,
                    redirect_status_code: None,
                    root_path: None,
                    default_if_url_is_dir: Some("/does-not-exist-mate"),
                    default_if_request_is_dir: None,
                    cgi_def: None,
                    list_directory: false,
                }),
            },
            Route {
                url_path: "/assets",
                methods: vec![
                    http::Method::GET,
                    http::Method::POST,
                    http::Method::PUT,
                    http::Method::PATCH,
                    http::Method::DELETE,
                ],
                handler: None,
                settings: Some(Settings {
                    http_redirections: None,
                    redirect_status_code: None,
                    root_path: None,
                    default_if_url_is_dir: None,
                    default_if_request_is_dir: None,
                    cgi_def: None,
                    list_directory: true,
                }),
            },
        ],
    }]
}
