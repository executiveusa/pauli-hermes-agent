const express = require("express");
const path = require("path");
const app = express();
const PORT = process.env.PORT || 3000;
const pub = path.join(__dirname, "public");

app.use(express.static(pub));
app.get("/healthz", (_req, res) => res.json({ status: "ok" }));
app.get("/dashboard", (_req, res) => res.sendFile(path.join(pub, "index.html")));
app.get("/", (_req, res) => res.sendFile(path.join(pub, "landing.html")));
app.get("*", (_req, res) => res.sendFile(path.join(pub, "landing.html")));

app.listen(PORT, "0.0.0.0", () => console.log(`Dashboard on :${PORT}`));
