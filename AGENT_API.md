# 🤖 AgentDesk Local Agent API Specification

The **Agent API** is a high-performance, stateless HTTP REST server built on `axum`. It binds by default to `127.0.0.1:7878` on the controlled host machine, allowing local or tunnelled AI agents (e.g., Claude Computer Use, OpenAI Operator, local autonomous LLM agents) to observe the display and inject precise keyboard and mouse actions.

---

## 🔒 Authentication

All endpoints (except `GET /health`) require bearer token authentication.

```http
Authorization: Bearer <AGENTDESK_TOKEN>
```

- Pass the token configured when starting `agentdesk-host` via `--token <value>` or environment variable `AGENTDESK_TOKEN`.
- If the token is missing, malformed, or invalid, the API returns `401 Unauthorized`:
```json
{
  "error": "Missing 'Authorization: Bearer <token>' header in request",
  "status": 401
}
```

---

## 📡 Endpoints Overview

| Method | Endpoint | Description | Auth Required |
|---|---|---|---|
| `GET` | `/health` | Health check & platform diagnosis | No |
| `GET` | `/screen-size` | Retrieve primary display width and height | Yes |
| `POST` | `/screenshot` | Capture display and return PNG as Base64 | Yes |
| `POST` | `/move` | Move mouse cursor to (x, y) coordinates | Yes |
| `POST` | `/click` | Click mouse button at (x, y) coordinates | Yes |
| `POST` | `/type` | Type text string into active window | Yes |
| `POST` | `/key` | Simulate key combo (e.g. `ctrl+shift+t`) | Yes |

---

## 📘 Detailed Endpoint Reference

### 1. `GET /health`
Verifies that the Agent API server is running and returns the detected operating system display platform.

#### Response `200 OK`
```json
{
  "status": "ok",
  "platform": "windows",
  "version": "0.1.0"
}
```
*Platform values*: `"linux-x11"`, `"linux-wayland"`, `"macos"`, `"windows"`, `"other"`.

#### Example cURL
```bash
curl -X GET http://127.0.0.1:7878/health
```

---

### 2. `GET /screen-size`
Retrieves current screen dimensions in pixels. Essential for agent coordinate normalization and bounding box scaling.

#### Response `200 OK`
```json
{
  "width": 1920,
  "height": 1080
}
```

#### Example cURL
```bash
curl -X GET http://127.0.0.1:7878/screen-size \
  -H "Authorization: Bearer agentdesk-secret-token"
```

---

### 3. `POST /screenshot`
Captures the display immediately, encodes the raw pixel frame into PNG format in memory, and returns the dimensions and base64-encoded PNG image string.

#### Request Body
```json
{
  "display": 0
}
```
*(Optional body. If omitted or `{}` passed, primary display `0` is captured).*

#### Response `200 OK`
```json
{
  "width": 1920,
  "height": 1080,
  "png_base64": "iVBORw0KGgoAAAANSUhEUgAAB4AAAA..."
}
```

#### Example cURL
```bash
curl -X POST http://127.0.0.1:7878/screenshot \
  -H "Authorization: Bearer agentdesk-secret-token" \
  -H "Content-Type: application/json" \
  -d '{}'
```

---

### 4. `POST /move`
Moves the mouse cursor to absolute screen coordinate `(x, y)`.

#### Request Body
```json
{
  "x": 450.0,
  "y": 280.0
}
```

#### Response `200 OK`
```json
{
  "success": true
}
```

#### Example cURL
```bash
curl -X POST http://127.0.0.1:7878/move \
  -H "Authorization: Bearer agentdesk-secret-token" \
  -H "Content-Type: application/json" \
  -d '{"x": 450.0, "y": 280.0}'
```

---

### 5. `POST /click`
Moves the cursor to `(x, y)` and simulates clicking a mouse button.

#### Request Body
```json
{
  "x": 800.0,
  "y": 450.0,
  "button": "left",
  "clicks": 1
}
```
- `button`: `"left"`, `"right"`, `"middle"` (default: `"left"`)
- `clicks`: Number of clicks, e.g. `1` for single click, `2` for double click (default: `1`)

#### Response `200 OK`
```json
{
  "success": true
}
```

#### Example cURL
```bash
curl -X POST http://127.0.0.1:7878/click \
  -H "Authorization: Bearer agentdesk-secret-token" \
  -H "Content-Type: application/json" \
  -d '{"x": 800.0, "y": 450.0, "button": "left", "clicks": 2}'
```

---

### 6. `POST /type`
Simulates typing UTF-8 text into whichever application currently has focus.

#### Request Body
```json
{
  "text": "cargo test --workspace\n"
}
```

#### Response `200 OK`
```json
{
  "success": true
}
```

#### Example cURL
```bash
curl -X POST http://127.0.0.1:7878/type \
  -H "Authorization: Bearer agentdesk-secret-token" \
  -H "Content-Type: application/json" \
  -d '{"text": "Hello, world!"}'
```

---

### 7. `POST /key`
Simulates pressing a combination of modifier and target keys simultaneously, then releasing them in reverse order.

#### Request Body
```json
{
  "combo": "ctrl+shift+t"
}
```

Supported modifiers:
- `ctrl`, `control`
- `shift`
- `alt`, `opt`, `option`
- `super`, `cmd`, `command`, `meta`, `win`

Supported target keys:
- `a`-`z`, `0`-`9`, `f1`-`f12`
- `enter`, `return`, `esc`, `escape`, `backspace`, `tab`, `space`, `delete`
- Arrow keys: `up`, `down`, `left`, `right`

#### Response `200 OK`
```json
{
  "success": true
}
```

#### Example cURL
```bash
curl -X POST http://127.0.0.1:7878/key \
  -H "Authorization: Bearer agentdesk-secret-token" \
  -H "Content-Type: application/json" \
  -d '{"combo": "ctrl+shift+t"}'
```
