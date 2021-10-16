import torch
import skimage.io as io
import PIL.Image

from model import model, preprocess, clip_model, tokenizer, PREFIX_LENGTH, DEVICE
from generate import generate2, generate_beam


def get_image_caption(image_url, prefix_length=PREFIX_LENGTH, use_beam_search=False):
    image = io.imread(image_url)
    pil_image = PIL.Image.fromarray(image)

    image = preprocess(pil_image).unsqueeze(0).to(DEVICE)
    with torch.no_grad():
        prefix = clip_model.encode_image(image).to(DEVICE, dtype=torch.float32)
        prefix_embed = model.clip_project(prefix).reshape(1, prefix_length, -1)
    if use_beam_search:
        generated_text_prefix = generate_beam(model, tokenizer, embed=prefix_embed)[0]
    else:
        generated_text_prefix = generate2(model, tokenizer, embed=prefix_embed)

    return generated_text_prefix
