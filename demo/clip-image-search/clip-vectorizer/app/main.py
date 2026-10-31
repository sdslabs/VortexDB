from io import BytesIO

from PIL import Image
from fastapi import FastAPI, File, UploadFile
from pydantic import BaseModel
import torch
from torch.nn.functional import normalize
from transformers import CLIPProcessor, CLIPModel

MODEL_ID = "openai/clip-vit-base-patch32"

device = "cuda" if torch.cuda.is_available() else "cpu"
print(f"Using device: {device}")

if device == "cuda":
    model = CLIPModel.from_pretrained(MODEL_ID, torch_dtype=torch.float16)
else:
    model = CLIPModel.from_pretrained(MODEL_ID)

processor = CLIPProcessor.from_pretrained(MODEL_ID, clean_up_tokenization_spaces=True)
model.to(device)
model.eval()

app = FastAPI()


class TextInput(BaseModel):
    text: str


@app.post("/vectors")
async def generate_text_embedding(text_input: TextInput):
    label_tokens = processor(
        text=text_input.text,
        padding=True,
        images=None,
        return_tensors='pt'
    ).to(device)

    with torch.no_grad():
        label_embeddings = model.get_text_features(**label_tokens)

    if hasattr(label_embeddings, 'pooler_output'):
        label_embeddings = label_embeddings.pooler_output
    elif hasattr(label_embeddings, 'last_hidden_state'):
        label_embeddings = label_embeddings.last_hidden_state[:, 0, :]

    label_embeddings = normalize(label_embeddings, p=2, dim=1)
    label_embeddings = label_embeddings.detach().cpu().tolist()
    return {"result": label_embeddings[0]}


@app.post("/vectors_img")
async def generate_image_embedding(file: UploadFile = File(...)):
    file_bytes = await file.read()
    image_stream = BytesIO(file_bytes)
    img = Image.open(image_stream).convert("RGB")

    image = processor(
        text=None,
        images=img,
        return_tensors='pt'
    ).to(device)['pixel_values']

    with torch.no_grad():
        image_embeddings = model.get_image_features(image)

    if hasattr(image_embeddings, 'pooler_output'):
        image_embeddings = image_embeddings.pooler_output
    elif hasattr(image_embeddings, 'last_hidden_state'):
        image_embeddings = image_embeddings.last_hidden_state[:, 0, :]

    image_embeddings = normalize(image_embeddings, p=2, dim=1)
    image_embeddings = image_embeddings.detach().cpu().tolist()
    return {"result": image_embeddings[0]} 

    