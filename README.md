# HNG13 Stage 1 - String Analyzer API

A simple RESTful API built with **Rust + Actix Web** for analyzing strings and storing their computed properties.

---

## 🚀 Base URL
Deployed on Railway:  
`https://<your-app-name>.up.railway.app`

---

## 🧠 Endpoints

| Method | Endpoint | Description |
|--------|-----------|-------------|
| POST | `/strings` | Analyze and store a string |
| GET | `/strings/{string_value}` | Retrieve analyzed string |
| GET | `/strings` | Get all analyzed strings |
| DELETE | `/strings/{string_value}` | Delete a string |

---

## 🧩 Example Usage

```bash
curl -X POST https://<your-app>.up.railway.app/strings \
  -H "Content-Type: application/json" \
  -d '{"value": "madam"}'
