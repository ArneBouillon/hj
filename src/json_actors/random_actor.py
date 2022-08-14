#!/usr/bin/python3

import json
import random
import subprocess
import sys


first_round = True
hearts_played = False

def score(card):
    if card[1] == 4:
        return 1
    if card[1] == 1 and card[0] == 12:
        return 13
    if card[1] == 3 and card[0] == 11:
        return -10
    return 0

def find_card():
    random.shuffle(cards)
    played_moves = message['played_moves']
    if len(played_moves):
        first_move = played_moves[0]
        for card in cards:
            if card[-1] == first_move[-1]:
                return card

        if first_round:
            for card in cards:
                if not score(card):
                    return card
        return cards[0]
    else:
        if first_round:
            return [2, 2]
        if not hearts_played:
            for card in cards:
                if card[-1] != 4:
                    return card
        return cards[0]

while True:
    char = None
    string = ""
    while char != "\n":
        char = sys.stdin.read(1)
        string += char

    message = json.loads(string)
    if message['message'] == 'initialize':
        pidx = message['pidx']
        cards = message['cards']
        string = '[]'
    elif message['message'] == 'play_card':
        card = find_card()
        cards.remove(card)
        string = json.dumps({'card': card})
    elif message['message'] == 'end_round':
        first_round = False
        if any(c[-1] == 4 for c in message['played_moves']):
            hearts_played = True
        string = '[]'
    elif message['message'] == 'end_game':
        string = '[]'

    string += '\n'
    with open('temp.txt', 'w+') as file:
        file.write(string)
    subprocess.run(['cat', 'temp.txt'])
