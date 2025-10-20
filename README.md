# HNG13 Stage 1 - String Analyzer API

A simple RESTful API built with **Rust + Actix Web** for analyzing strings and storing their computed properties. This API supports storing strings, querying them (including natural language filters), and deleting them.

---

## 🚀 Base URL

Deployed on Railway:
`hng13-stage1-string-analyzer-service-production.up.railway.app`

---

## 🧠 Endpoints

| Method | Endpoint                                            | Description                                   |
| ------ | --------------------------------------------------- | --------------------------------------------- |
| POST   | `/strings`                                          | Analyze and store a string                    |
| GET    | `/strings/{string_value}`                           | Retrieve an analyzed string                   |
| GET    | `/strings`                                          | Get all analyzed strings                      |
| GET    | `/strings/filter-by-natural-language?query=<query>` | Filter strings using natural language queries |
| DELETE | `/strings/{string_value}`                           | Delete a string                               |

---

## ⚙️ Setup Instructions

### 1. Clone the repository

```bash
git clone https://github.com/arabson99/hng13-stage1-string-analyzer-service.git
cd hng13-stage1-string-analyzer-service

### 2. Install Rust

Make sure Rust is installed (1.83+ recommended):

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### 3. Install system dependencies (Debian/Ubuntu)

```bash
sudo apt update
sudo apt install -y libssl-dev ca-certificates
```

### 4. Build the project

```bash
cargo build --release
```

### 5. Run locally

```bash
cargo run --release
```

The API will run on `http://127.0.0.1:8080` by default.

---

## 📦 Dependencies

* Rust (1.83+)
* Actix Web
* Serde / Serde JSON
* SHA2 (for string hashing)
* System libraries: `libssl-dev`, `ca-certificates`

---

## 🌐 Environment Variables

* `PORT` – Optional. Default is `8080`.

---

## 🧩 Example Usage

### Add a string

```bash
curl -X POST http://127.0.0.1:8080/strings \
  -H "Content-Type: application/json" \
  -d '{"value": "madam"}'
```

### Retrieve a string

```bash
curl http://127.0.0.1:8080/strings/madam
```

### List all strings

```bash
curl http://127.0.0.1:8080/strings
```

### Delete a string

```bash
curl -X DELETE http://127.0.0.1:8080/strings/madam
```

### Filter by natural language

```bash
curl "http://127.0.0.1:8080/strings/filter-by-natural-language?query=all%20single%20word%20palindromic%20strings"
```

---

## 🔗 GitHub Repository

[https://github.com/arabson99/hng13-stage1-string-analyzer-service](https://github.com/arabson99/hng13-stage1-string-analyzer-service.git)
