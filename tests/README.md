Add tests here for:
- WebSocket handshake and initial `mso` response.
- Fragmented/large text frames and malformed JSON.
- `msoupdate` patch application, including nested paths and arrays.
- Reconnect behavior and state reset.
- Command serialization, confirmation and no implicit retries.
- Couch SDK contract/admission tests once the adapter exists.
