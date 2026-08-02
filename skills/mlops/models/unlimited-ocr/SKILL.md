---
name: unlimited-ocr
description: "Unlimited-OCR: Baidu's long-horizon document/image OCR and layout parsing, multi-page + PDF support."
version: 1.0.0
author: Orchestra Research
license: MIT
dependencies: [torch==2.10.0, torchvision==0.25.0, transformers==4.57.1, Pillow==12.1.1, einops==0.8.2, pymupdf==1.27.2.2]
platforms: [linux]
metadata:
  hermes:
    tags: [Multimodal, OCR, Document Parsing, Vision-Language, Scanning, PDF]

---

# Unlimited-OCR: Document & Image Scanning

Guide to running Baidu's Unlimited-OCR — a vision-language model built on DeepSeek-OCR for
"one-shot long-horizon parsing": extracting text and layout from single images, multi-page
document sets, and PDFs in one pass.

Repo: https://github.com/baidu/Unlimited-OCR (MIT license)

## When to use

**Use Unlimited-OCR when:**
- Scanning a photo or screenshot of a document and need the text extracted with layout
- Parsing a multi-page scanned document or a PDF that needs OCR (not just text-layer extraction)
- The document has complex layout — tables, forms, mixed columns — where naive text extraction
  fails
- You want bounding-box-annotated output (`<|det|>` markers) alongside the extracted text, not
  just plain text

**Use alternatives instead:**
- **`pdf` skill** — if the PDF already has a text layer, or you just need basic scanned-PDF OCR
  without a dedicated GPU model; much lighter weight for simple cases
- **Cloud OCR APIs** (Baidu Cloud OCR service, Google Vision, AWS Textract) — if no local/remote
  GPU is available; Unlimited-OCR requires an NVIDIA GPU
- **General vision-language models via the agent's own image understanding** — for a quick,
  approximate read of a single image where exact bounding boxes and dense-document accuracy
  don't matter

**Hardware requirement — check this first:** Unlimited-OCR needs an NVIDIA GPU (tested on CUDA
12.9, bfloat16 precision). Before doing anything else, verify one is available:

```bash
nvidia-smi
```

If there is no GPU in this environment, tell the user and fall back to a cloud OCR API or the
`pdf` skill instead of trying to run this locally.

## Setup

```bash
git clone https://github.com/baidu/Unlimited-OCR.git
cd Unlimited-OCR
pip install torch==2.10.0 torchvision==0.25.0 transformers==4.57.1 \
    Pillow==12.1.1 matplotlib==3.10.8 einops==0.8.2 addict==2.4.0 \
    easydict==1.13 pymupdf==1.27.2.2 psutil==7.2.2
```

Tested with Python 3.12.3 + CUDA 12.9. Model checkpoints are pulled automatically from
Hugging Face (`baidu/Unlimited-OCR`) on first run, or available on ModelScope
(`PaddlePaddle/Unlimited-OCR`).

## Core concepts

Two processing modes, pick based on input:

| Mode | `base_size` / `image_size` | Use for |
|------|------|---------|
| **Gundam** | `base_size=1024`, `image_size=640` | Single images |
| **Base** | `image_size=1024` | Multi-page documents, PDFs |

- **Max context**: 32,768 tokens — long/dense documents may need chunking across pages
- **Repeat suppression**: `no_repeat_ngram_size=35` guards against the model looping on
  repetitive table/list content
- **Output markers**: extracted text can include `<|det|>` bounding-box annotations inline —
  strip these if the user just wants plain text, keep them if they need layout coordinates

## Basic usage (Transformers)

### Single image

```python
from transformers import AutoModel, AutoTokenizer

tokenizer = AutoTokenizer.from_pretrained("baidu/Unlimited-OCR", trust_remote_code=True)
model = AutoModel.from_pretrained("baidu/Unlimited-OCR", trust_remote_code=True).cuda().eval()

result = model.infer(
    tokenizer,
    prompt="<image>document parsing.",
    image_file="your_image.jpg",
    base_size=1024,
    image_size=640,
    crop_mode=True,
)
```

### Multi-page

```python
result = model.infer_multi(
    tokenizer,
    prompt="<image>Multi page parsing.",
    image_files=["page1.png", "page2.png"],
    image_size=1024,
)
```

### PDF input

Unlimited-OCR takes images, not PDFs directly — rasterize pages first with PyMuPDF, then feed
the resulting images to `infer_multi`:

```python
import fitz  # pymupdf

doc = fitz.open("document.pdf")
image_files = []
for i, page in enumerate(doc):
    pix = page.get_pixmap(dpi=200)
    path = f"page_{i}.png"
    pix.save(path)
    image_files.append(path)
```

## Batch inference via `infer.py`

For scanning a directory of images (or a PDF) without writing custom Python, use the repo's
CLI script:

```bash
python infer.py \
    --image_dir ./examples/images \
    --output_dir ./outputs \
    --model_dir baidu/Unlimited-OCR \
    --image_mode gundam \
    --concurrency 8 \
    --gpu 0
```

For a PDF instead of an image directory:

```bash
python infer.py \
    --pdf ./examples/document.pdf \
    --output_dir ./outputs \
    --model_dir baidu/Unlimited-OCR \
    --image_mode base \
    --concurrency 8 \
    --gpu 0
```

Key flags: `--image_dir`, `--pdf` (mutually exclusive inputs), `--output_dir`, `--model_dir`,
`--image_mode` (`gundam` for single images, `base` for multi-page/PDF), `--concurrency`, `--gpu`.

## Server deployment (higher throughput)

For repeated/production scanning workloads, run the model behind a server instead of loading
it per-invocation.

### vLLM

```bash
docker pull vllm/vllm-openai:unlimited-ocr
# Hopper-class GPUs:
docker pull vllm/vllm-openai:unlimited-ocr-cu129
```

Full deployment recipe: https://recipes.vllm.ai/baidu/Unlimited-OCR

### SGLang

```bash
python -m sglang.launch_server \
    --model baidu/Unlimited-OCR \
    --context-length 32768 \
    --port 10000 \
    2>&1 | tee ./log/sglang_server.log
```

Once the server is up, point `infer.py`'s batch mode at it (`--server_log` tails the launch log
to confirm readiness before sending requests).

## Workflow

1. **Check hardware**: `nvidia-smi` — if no GPU, stop and recommend a cloud OCR fallback.
2. **Classify input**: single image → Gundam mode; multi-page or PDF → Base mode (rasterize PDF
   pages first via PyMuPDF).
3. **Run inference**: use `infer.py` for a directory/PDF of inputs, or the Python API for a
   single one-off image.
4. **Post-process**: strip `<|det|>` markers if the user wants plain text; keep them if they
   need bounding boxes for downstream layout use.
5. **Verify**: spot-check output against the source image/page for garbled or truncated text,
   especially near the 32,768-token context ceiling on dense multi-page documents.

## Common issues

| Issue | Solution |
|-------|----------|
| No CUDA device found | Requires NVIDIA GPU; use a cloud OCR API or the `pdf` skill instead |
| Repeated/looping output on tables | Confirm `no_repeat_ngram_size=35` is set; reduce input density per page |
| Output truncated on long documents | Split into fewer pages per `infer_multi` call — 32,768 token context ceiling |
| PDF pages come out blank/blurry | Increase rasterization `dpi` (e.g. 200-300) when calling `get_pixmap` |
| Wrong mode used | Gundam is for single images; Base is for multi-page/PDF — mismatched mode degrades accuracy |

## Resources

- **GitHub**: https://github.com/baidu/Unlimited-OCR
- **Hugging Face**: https://huggingface.co/baidu/Unlimited-OCR
- **ModelScope**: https://modelscope.cn/models/PaddlePaddle/Unlimited-OCR
- **vLLM recipe**: https://recipes.vllm.ai/baidu/Unlimited-OCR
- **Paper**: https://arxiv.org/abs/2606.23050
- **License**: MIT
