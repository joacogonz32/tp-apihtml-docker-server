// COMANDOS DE TERMINAL PARA EJECUTAR ESTE PROYECTO CON DOCKER
//
// 1. Crear la red de Docker (si no existe):
//    docker network create mired
//
// 2. Levantar el contenedor de MySQL conectado a la red:
//    docker run -d --name db-mysql --network mired \
//      --env-file ../db.env \
//      -e MYSQL_ROOT_PASSWORD=alumnoipm \
//      -e MYSQL_DATABASE=tp-proyecto-docker_db \
//      -p 3306:3306 \
//      mysql:8.0
//
// 3. Construir la imagen del backend Actix Web:
//    docker build -t actix-backend .
//
// 4. Correr el contenedor del backend conectado a la misma red:
//    docker run -d --name actix-app --network mired \
//      --env-file ../db.env \
//      -p 8080:8080 \
//      actix-backend
//

use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};
use sqlx::mysql::MySqlPoolOptions;
use sqlx::{FromRow, MySqlPool};
use std::env;

// MODELOS 

#[derive(Serialize, Deserialize)]
struct CreateItem {
    nombre: String,
}

#[derive(Serialize, FromRow)]
struct Item {
    id: i32,
    nombre: String,
    created_at: Option<chrono::NaiveDateTime>,
}

#[derive(Serialize)]
struct DbStatusOk {
    status: String,
    time: String,
}

#[derive(Serialize)]
struct DbStatusErr {
    status: String,
    error: String,
}

// ESTADO COMPARTIDO 

struct AppState {
    pool: MySqlPool,
}

// HANDLERS

#[get("/health")]
async fn health() -> impl Responder {
    HttpResponse::Ok().body("OK")
}

#[get("/db-status")]
async fn db_status(data: web::Data<AppState>) -> impl Responder {
    let result = sqlx::query_scalar::<_, String>("SELECT NOW()")
        .fetch_one(&data.pool)
        .await;

    match result {
        Ok(time) => HttpResponse::Ok().json(DbStatusOk {
            status: "connected".to_string(),
            time,
        }),
        Err(e) => HttpResponse::InternalServerError().json(DbStatusErr {
            status: "disconnected".to_string(),
            error: e.to_string(),
        }),
    }
}

#[post("/items")]
async fn create_item(
    data: web::Data<AppState>,
    body: web::Json<CreateItem>,
) -> impl Responder {
    if body.nombre.is_empty() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "El campo 'nombre' es obligatorio"
        }));
    }

    let result = sqlx::query("INSERT INTO items (nombre) VALUES (?)")
        .bind(&body.nombre)
        .execute(&data.pool)
        .await;

    match result {
        Ok(res) => {
            let id = res.last_insert_id();
            HttpResponse::Created().json(serde_json::json!({
                "id": id,
                "nombre": body.nombre
            }))
        }
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e.to_string()
        })),
    }
}

#[get("/items")]
async fn get_items(data: web::Data<AppState>) -> impl Responder {
    let result = sqlx::query_as::<_, Item>("SELECT id, nombre, created_at FROM items")
        .fetch_all(&data.pool)
        .await;

    match result {
        Ok(items) => HttpResponse::Ok().json(items),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e.to_string()
        })),
    }
}

// MAIN 

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let db_host = env::var("DB_HOST").expect("DB_HOST no definida");
    let db_user = env::var("DB_USER").expect("DB_USER no definida");
    let db_password = env::var("DB_PASSWORD").expect("DB_PASSWORD no definida");
    let db_name = env::var("DB_NAME").expect("DB_NAME no definida");

    let database_url = format!(
        "mysql://{}:{}@{}/{}",
        db_user, db_password, db_host, db_name
    );

    let pool = MySqlPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("No se pudo conectar a MySQL");

    // Crear tabla si no existe
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS items (
            id INT AUTO_INCREMENT PRIMARY KEY,
            nombre VARCHAR(255) NOT NULL,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )",
    )
    .execute(&pool)
    .await
    .expect("No se pudo crear la tabla items");

    println!("Servidor escuchando en 0.0.0.0:8080");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(AppState { pool: pool.clone() }))
            .service(health)
            .service(db_status)
            .service(create_item)
            .service(get_items)
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
