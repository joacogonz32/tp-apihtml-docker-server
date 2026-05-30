# COMANDOS EN LA TERMINAL PARA EJECUTAR PROYECTO CON DOCKER
# 1. Crear la red de Docker:
#    docker network create mired

# 2. Levantar el contenedor de MySQL conectado a la red:
#    docker run -d --name db-mysql --network mired \
#      --env-file ../db.env \
#      -e MYSQL_ROOT_PASSWORD=alumnoipm \
#      -e MYSQL_DATABASE=tp-proyecto-docker_db \
#      -p 3306:3306 \
#      mysql:8.0

# 3. Construir la imagen del backend Flask:
#    docker build -t flask-backend .

# 4. Correr el contenedor del backend conectado a la misma red:
#    docker run -d --name flask-app --network mired \
#      --env-file ../db.env \
#      -p 5000:5000 \
#      flask-backend
#

import os
import socket
from pathlib import Path
from flask import Flask, request, jsonify
import mysql.connector

app = Flask(__name__)


def load_env_file():
    """Carga db.env para ejecuciones locales sin pisar variables ya definidas."""
    candidate_paths = [
        Path(__file__).resolve().parent / ".env",
        Path(__file__).resolve().parent.parent / "db.env",
    ]

    for env_path in candidate_paths:
        if not env_path.exists():
            continue

        for raw_line in env_path.read_text(encoding="utf-8").splitlines():
            line = raw_line.strip()
            if not line or line.startswith("#") or "=" not in line:
                continue

            key, value = line.split("=", 1)
            os.environ.setdefault(key.strip(), value.strip())
        return
load_env_file()

def get_db_host():
    """Usa DB_HOST y cae a localhost cuando el hostname solo existe dentro de Docker."""
    db_host = os.environ["DB_HOST"]
    try:
        socket.getaddrinfo(db_host, int(os.environ.get("DB_PORT", "3306")))
        return db_host
    except socket.gaierror:
        return "127.0.0.1"

def get_db_connection():
    """Crea y retorna una conexión a la base de datos MySQL"""
    return mysql.connector.connect(
        host=get_db_host(),
        user=os.environ["DB_USER"],
        password=os.environ["DB_PASSWORD"],
        database=os.environ["DB_NAME"],
        port=int(os.environ.get("DB_PORT", "3306")),
    )

def init_db():
    """Crea la tabla 'items' si no existe"""
    try:
        conn = get_db_connection()
        cursor = conn.cursor()
        cursor.execute(
            """
            CREATE TABLE IF NOT EXISTS items (
                id INT AUTO_INCREMENT PRIMARY KEY,
                nombre VARCHAR(255) NOT NULL,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )
            """
        )
        conn.commit()
        cursor.close()
        conn.close()
    except Exception as e:
        print(f"[init_db] No se pudo inicializar la tabla: {e}")

# ENDPOINTS

@app.route("/health", methods=["GET"])
def health():
    return "OK", 200

@app.route("/db-status", methods=["GET"])
def db_status():
    try:
        conn = get_db_connection()
        cursor = conn.cursor()
        cursor.execute("SELECT NOW();")
        row = cursor.fetchone()
        cursor.close()
        conn.close()
        return jsonify({"status": "connected", "time": str(row[0])}), 200
    except Exception as e:
        return jsonify({"status": "disconnected", "error": str(e)}), 500

@app.route("/items", methods=["POST"])
def create_item():
    body = request.get_json(force=True)
    nombre = body.get("nombre")
    if not nombre:
        return jsonify({"error": "El campo 'nombre' es obligatorio"}), 400

    try:
        conn = get_db_connection()
        cursor = conn.cursor()
        cursor.execute("INSERT INTO items (nombre) VALUES (%s)", (nombre,))
        conn.commit()
        item_id = cursor.lastrowid
        cursor.close()
        conn.close()
        return jsonify({"id": item_id, "nombre": nombre}), 201
    except Exception as e:
        return jsonify({"error": str(e)}), 500

@app.route("/items", methods=["GET"])
def get_items():
    try:
        conn = get_db_connection()
        cursor = conn.cursor(dictionary=True)
        cursor.execute("SELECT id, nombre, created_at FROM items")
        rows = cursor.fetchall()
        cursor.close()
        conn.close()
        # Convertir datetime a string para JSON
        for row in rows:
            row["created_at"] = str(row["created_at"])
        return jsonify(rows), 200
    except Exception as e:
        return jsonify({"error": str(e)}), 500

# MAIN 

if __name__ == "__main__":
    init_db()
    app.run(host="0.0.0.0", port=5000)
