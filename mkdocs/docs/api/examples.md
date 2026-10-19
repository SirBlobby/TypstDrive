# Examples

Replace `https://typst.example.com` with the address of your TypstDrive instance and `td_your_api_key_here` with one of your keys.

## cURL

=== "PNG"

    ```bash
    curl -X POST https://typst.example.com/v1/render \
      -H "Authorization: Bearer td_your_api_key_here" \
      -H "Content-Type: application/json" \
      -d '{"code":"#set page(width: 200pt, height: 80pt)\nHello, *World*!","format":"png"}' \
      --output hello.png
    ```

=== "PDF"

    ```bash
    curl -X POST https://typst.example.com/v1/render \
      -H "Authorization: Bearer td_your_api_key_here" \
      -H "Content-Type: application/json" \
      -d '{"code":"= My Report\n\nSome body text.","format":"pdf"}' \
      --output report.pdf
    ```

=== "HTML"

    ```bash
    curl -X POST https://typst.example.com/v1/render \
      -H "Authorization: Bearer td_your_api_key_here" \
      -H "Content-Type: application/json" \
      -d '{"code":"= My Report\n\nSome body text.","format":"html"}' \
      --output report.html
    ```

## JavaScript and TypeScript

```javascript
const response = await fetch('https://typst.example.com/v1/render', {
  method: 'POST',
  headers: {
    Authorization: 'Bearer td_your_api_key_here',
    'Content-Type': 'application/json',
  },
  body: JSON.stringify({
    code: '#set page(width: 200pt, height: 80pt)\nHello, *World*!',
    format: 'png',
  }),
});

if (!response.ok) throw new Error(await response.text());

const image = await response.blob();
document.querySelector('img').src = URL.createObjectURL(image);
```

!!! warning
    Run this from a server or a trusted tool. Putting an API key in a public web page exposes it to everyone who visits.

## Python

```python
import httpx

response = httpx.post(
    "https://typst.example.com/v1/render",
    headers={"Authorization": "Bearer td_your_api_key_here"},
    json={"code": "= My Report\n\nSome body text.", "format": "pdf"},
)
response.raise_for_status()

with open("report.pdf", "wb") as output:
    output.write(response.content)
```

## Python with inline files

```python
import base64
import httpx

with open("logo.png", "rb") as logo:
    logo_data = base64.b64encode(logo.read()).decode()

response = httpx.post(
    "https://typst.example.com/v1/render",
    headers={"Authorization": "Bearer td_your_api_key_here"},
    json={
        "code": '#set page(width: 300pt, height: 200pt)\n#image("logo.png", width: 80pt)\n= My Report\nSome body text.',
        "format": "png",
        "files": [{"name": "logo.png", "data": logo_data}],
    },
)
response.raise_for_status()

with open("output.png", "wb") as output:
    output.write(response.content)
```

## Handling compile errors

```python
response = httpx.post(url, headers=headers, json=payload)

if response.status_code == 422:
    problem = response.json()
    for detail in problem["details"]:
        print(f'{detail["severity"]} at {detail["line"]}:{detail["column"]}: {detail["message"]}')
elif response.status_code == 429:
    print("Rate limit or daily quota reached, try again later.")
else:
    response.raise_for_status()
```
