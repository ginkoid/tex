# plusbot-latex

Fast and secure LaTeX renderer service for [plusbot](https://github.com/ginkoid/plusbot)

## Protocol

The renderer communicates over TCP using a request-response structure.

### Request structure
| Type     | Offset | Name | Description                    |
| -------- | ------ | ---- | ------------------------------ |
| uint32be | 0      | type | [Request type](#request-types) |
| uint32be | 4      | len  | Byte length of LaTeX document  |
| bytes    | 8      | data | LaTeX document of length `len` |

### Response structure
| Type     | Offset | Name | Description                      |
| -------- | ------ | ---- | -------------------------------- |
| uint32be | 0      | type | [Response type](#response-types) |
| uint32be | 4      | len  | Byte length of response content  |
| bytes    | 8      | data | Response content of length `len` |

### Request types
| Value | Description     |
| ----- | --------------- |
| 0     | LaTeX rendering |

### Response types
| Value | Description                            |
| ----- | -------------------------------------- |
| 0     | Success: PNG in `data`                 |
| 1     | Error from PDFLaTeX: Message in `data` |
