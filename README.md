# plusbot-latex

Fast and secure LaTeX renderer service for [plusbot](https://github.com/ginkoid/plusbot)

## Protocol

The renderer communicates over TCP. After connecting, send a single LaTeX document. The body is followed by a single `uint32be` [response type](#response-type) value.

### Response types
| Value | Description                      |
| ----- | -------------------------------- |
| 0     | Success: PNG in body             |
| 1     | Error from PDFLaTeX: Log in body |
| 2     | Error from GhostScript           |
| 3     | Internal error                   |
