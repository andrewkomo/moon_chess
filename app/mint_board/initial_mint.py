import random
from board_generator import create_board
import numpy as np

MIN_NODE_SIDE = 1
MAX_NODE_SIDE = 4
COLOR_SHIFT = 0.3

NUM_BOARDS = 500

border_colors = [(0,0,128),(128,0,0),(0,0,0),(120,80,50),(70,70,70)]
border_color_weights = np.cumsum([0.075,0.025,0.3,0.4,0.2])
dark_colors = [(0,0,0),(180,140,100),(125,75,140),(105,125,70),(140,160,175)]
light_colors = [(255,255,255),(240,215,180),(190,180,200),(255,255,220),(220,225,230)]
color_weights = np.cumsum([0.5,0.2,0.025,0.15,0.125])
p_edges = []
for i in range(0,105,5):
    p_edges += [i/100]*(21-i//5)
node_weights = np.cumsum([8/15,4/15,2/15,1/15])
border_style_weights = np.cumsum([0.8,0.15,0.05])

past_hashes = set([0])


def weighted_choice(weights):
    draw = random.random()
    choice = 0
    while weights[choice] < draw:
        choice += 1
    return choice


for i in range(NUM_BOARDS):
    new_hash = 0
    while new_hash in past_hashes:
        print(i)
        border_color = border_colors[weighted_choice(border_color_weights)]

        square_color_choice = weighted_choice(color_weights)
        dark_color = dark_colors[square_color_choice]
        light_color = light_colors[square_color_choice]

        nodes_per_side = weighted_choice(node_weights) + 1 # Between 1 and 4

        diff = [x-y for (x,y) in zip(light_color,dark_color)]
        dark_line = tuple(int(COLOR_SHIFT*x+y) for (x,y) in zip(diff,dark_color))
        light_line = tuple(int(y-COLOR_SHIFT*x) for (x,y) in zip(diff,dark_color))

        p_edge = random.choice(p_edges)

        border_style = weighted_choice(border_style_weights)

        new_hash = create_board(border_color,dark_color,light_color,dark_line,light_line,nodes_per_side,p_edge,border_style,'app/mint_board/assets/',i,f'#{i+1}/{NUM_BOARDS} of the genesis Shachess Chessboards')
    past_hashes.add(new_hash)