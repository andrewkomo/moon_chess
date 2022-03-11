from PIL import Image, ImageDraw, ImageFont
import random
import textwrap

SQUARE_DIM = 128
LINE_WIDTH = 1
MIN_NODE_SIDE = 1
MAX_NODE_SIDE = 4
BORDER_SIZE = 64
COLOR_SHIFT = 0.3
P_POCKMARK = 0.05
ROYALTY_FEE = 250
TARGET_WALLET = "DhX4pf9j72hpkJPxmRVbfx8Rg95zczx4y8FLihoeGKeK"

def border_shift(L):
    return list(map(lambda x:x+BORDER_SIZE, L))

def generate_gradient(color1, color2, length):
    """Generate a vertical gradient."""
    base = Image.new('RGB', (length, length), color1)
    top = Image.new('RGB', (length, length), color2)
    mask = Image.new('L', (length, length))
    mask_data = []
    for y in range(length):
        mask_data.extend([int(255 * (y / length))] * length)
    mask.putdata(mask_data)
    base.paste(top, (0, 0), mask)
    return base

def create_board(border_color,dark_color,light_color,dark_line,light_line,nodes_per_side,p_edge,border_style,save_path,number):

    if border_style == 0 or border_style == 1:
        im = Image.new(mode="RGB", size=(SQUARE_DIM*8+BORDER_SIZE*2, SQUARE_DIM*8+BORDER_SIZE*2),color=border_color)
    else:
        im = generate_gradient(border_color,(0,0,0),SQUARE_DIM*8+BORDER_SIZE*2)
    draw = ImageDraw.Draw(im)
    
    if border_style == 1:
        for i in range(im.size[0]):
            for j in range(im.size[1]):
                if random.random() < P_POCKMARK:
                    draw.point((i,j),fill=(255,255,255))

    for i in range(0,8,2):
        for j in range(0,8,2):
            draw.rectangle(border_shift([i*SQUARE_DIM, j*SQUARE_DIM, (i+1)*SQUARE_DIM, (j+1)*SQUARE_DIM]),fill=light_color)
            draw.rectangle(border_shift([(i+1)*SQUARE_DIM, j*SQUARE_DIM, (i+2)*SQUARE_DIM, (j+1)*SQUARE_DIM]),fill=dark_color)
            draw.rectangle(border_shift([(i+1)*SQUARE_DIM, (j+1)*SQUARE_DIM, (i+2)*SQUARE_DIM, (j+2)*SQUARE_DIM]),fill=light_color)
            draw.rectangle(border_shift([i*SQUARE_DIM, (j+1)*SQUARE_DIM, (i+1)*SQUARE_DIM, (j+2)*SQUARE_DIM]),fill=dark_color)

    
    nodes = []
    for i in range(nodes_per_side):
        loc = SQUARE_DIM/(nodes_per_side+1)*(i+1)
        nodes += [(loc,0),(0,loc),(loc,SQUARE_DIM-1),(SQUARE_DIM-1,loc)]
    nodes += [(0,0),(0,SQUARE_DIM-1),(SQUARE_DIM-1,0),(SQUARE_DIM-1,SQUARE_DIM-1)]

    for i in range(len(nodes)):
        for j in range(i+1,len(nodes)):
            if random.random() < p_edge:
                for m in range(0,8,2):
                    for n in range(0,8,2):
                        draw.line(border_shift([nodes[i][0]+m*SQUARE_DIM, nodes[i][1]+n*SQUARE_DIM, nodes[j][0]+m*SQUARE_DIM, nodes[j][1]+n*SQUARE_DIM]), fill=light_line, width=LINE_WIDTH)
                        draw.line(border_shift([nodes[i][0]+(m+1)*SQUARE_DIM, nodes[i][1]+(n+1)*SQUARE_DIM, nodes[j][0]+(m+1)*SQUARE_DIM, nodes[j][1]+(n+1)*SQUARE_DIM]), fill=light_line, width=LINE_WIDTH)
            if random.random() < p_edge:
                for m in range(0,8,2):
                    for n in range(0,8,2):
                        draw.line(border_shift([nodes[i][0]+(m+1)*SQUARE_DIM, nodes[i][1]+n*SQUARE_DIM, nodes[j][0]+(m+1)*SQUARE_DIM, nodes[j][1]+n*SQUARE_DIM]), fill=dark_line, width=LINE_WIDTH)
                        draw.line(border_shift([nodes[i][0]+m*SQUARE_DIM, nodes[i][1]+(n+1)*SQUARE_DIM, nodes[j][0]+m*SQUARE_DIM, nodes[j][1]+(n+1)*SQUARE_DIM]), fill=dark_line, width=LINE_WIDTH)

    font = ImageFont.truetype("arial.ttf", 24)
    for i in range(8):
        draw.text((BORDER_SIZE+8*SQUARE_DIM+BORDER_SIZE/2-7, BORDER_SIZE+i*SQUARE_DIM+SQUARE_DIM/2-15), str(8-i), font=font)
        draw.text((BORDER_SIZE+i*SQUARE_DIM+SQUARE_DIM/2-7, BORDER_SIZE+8*SQUARE_DIM+BORDER_SIZE/2-15), chr(ord('a')+i), font=font)

    im.save(f'{save_path}{number}.png')
    with open(f'{save_path}{number}.json','w') as f:
        f.write(textwrap.dedent(f'''
        {{
            "name": "Moon Chessboard #{number+1}",
            "symbol": "CHESS",
            "description": "TBA",
            "seller_fee_basis_points": {ROYALTY_FEE},
            "image": "{number}.png",
            "attributes": [
                {{"trait_type": "border-color", "value": {list(border_color)}}},
                {{"trait_type": "dark-color", "value": {list(dark_color)}}}, 
                {{"trait_type": "light-color", "value": {list(light_color)}}},
                {{"trait_type": "dark-line", "value": {list(dark_line)}}},
                {{"trait_type": "light-line", "value": {list(light_line)}}},
                {{"trait_type": "nodes-per-side", "value": {list(nodes_per_side)}}},
                {{"trait_type": "p-edge", "value": {p_edge}}},
                {{"trait_type": "border-style", "value": {border_style}}}
            ],
            "properties": {{
                "creators": [{{"address": "{TARGET_WALLET}", "share": 100}}],
                "files": [{{"uri": "{number}.png", "type": "image/png"}}]
            }}
        }}
        '''))

border_colors = [(0,0,128),(128,0,0),(0,0,0),(120,80,50),(70,70,70)]
dark_colors = [(0,0,0),(180,140,100),(125,75,140),(105,125,70),(140,160,175)]
light_colors = [(255,255,255),(240,215,180),(190,180,200),(255,255,220),(220,225,230)]
p_edge = 0.6

for i in range(20):
    dark_color = random.choice(dark_colors)
    light_color = random.choice(light_colors)
    border_color = random.choice(border_colors)

    diff = [x-y for (x,y) in zip(light_color,dark_color)]
    dark_line = tuple(int(COLOR_SHIFT*x+y) for (x,y) in zip(diff,dark_color))
    light_line = tuple(int(y-COLOR_SHIFT*x) for (x,y) in zip(diff,dark_color))

    nodes_per_side = random.randint(MIN_NODE_SIDE,MAX_NODE_SIDE)

    create_board(border_color,dark_color,light_color,dark_line,light_line,nodes_per_side,p_edge,random.randint(0,2),'programs/mint_board/assets/',i)