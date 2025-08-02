#!/usr/bin/env python3

from PIL import Image
import sys

try:
    # Open the original image
    img = Image.open('app_icon.png')
    
    # Convert to RGBA if not already
    if img.mode != 'RGBA':
        img = img.convert('RGBA')
    
    # Save with alpha channel
    img.save('app_icon_rgba.png', 'PNG')
    print('Successfully converted to RGBA format')
    
    # Verify the conversion
    test_img = Image.open('app_icon_rgba.png')
    print(f'New image mode: {test_img.mode}')
    print(f'New image size: {test_img.size}')
    
except ImportError:
    print('PIL not available - please install Pillow: pip install Pillow')
    sys.exit(1)
except Exception as e:
    print(f'Error: {e}')
    sys.exit(1)