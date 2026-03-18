from PIL import Image, ImageDraw

def mask_squircle(img_path, output_path):
    print(f"Opening {img_path}")
    img = Image.open(img_path).convert("RGBA")
    
    # 1024 to 760 (132 margin)
    box = (132, 132, 1024 - 132, 1024 - 132)
    cropped = img.crop(box)
    
    # Create mask
    mask = Image.new('L', cropped.size, 0)
    draw = ImageDraw.Draw(mask)
    draw.rounded_rectangle((0, 0, cropped.size[0], cropped.size[1]), radius=150, fill=255)
    
    # Apply mask
    result = Image.new('RGBA', cropped.size)
    result.paste(cropped, (0,0), mask)
    
    result.save(output_path)
    print(f"Saved {output_path}")

mask_squircle('/Users/kpas/.gemini/antigravity/brain/b9c7fbe6-0ee8-455e-a128-c7478115fafc/rsendmail_app_icon_1773797263789.png', 'app-icon.png')
