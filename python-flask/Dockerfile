# Dockerfile: Python + Flask
FROM python:3.12-slim
WORKDIR /app

# Capa de dependencias (revisa si requerimientos.txt no cambia)
COPY requerimientos.txt .
RUN pip install --no-cache-dir -r requerimientos.txt

# Copia el codigo fuente
COPY . .

EXPOSE 5000

CMD ["python", "app.py"]
