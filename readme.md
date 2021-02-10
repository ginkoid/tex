# plusbot-latex

Secure LaTeX renderer service for [plusbot](https://github.com/ginkoid/plusbot)

```sh
sysctl -w kernel.unprivileged_userns_clone=1 # debian only
docker-compose up
```

## API

### Request structure
| Type     | Offset | Name | Description                    |
| -------- | ------ | ---- | ------------------------------ |
| uint32le | 0      | len  | Byte length of LaTeX document  |
| bytes    | 4      | data | LaTeX document of length `len` |

### Response structure
| Type     | Offset | Name | Description                      |
| -------- | ------ | ---- | -------------------------------- |
| uint32le | 0      | code | [Response code](#response-codes) |
| bytes    | 4      | data | Response content                 |

### Response codes
| Value | Description                            |
| ----- | -------------------------------------- |
| 0     | Success: PNG in `data`                 |
| 1     | Error from PDFLaTeX: Message in `data` |
