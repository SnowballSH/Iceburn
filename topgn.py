import chess.pgn

data = """
d2d4 d7d5 e2e4 d5e4 b1d2 d8d4 d2e4 d4b2 c1b2 b8c6 b2g7 f8g7 g1f3 g7a1 d1a1 g8f6 e4f6 e7f6 f3d4 c6d4"""

data = data.strip().split()

game = chess.pgn.Game()
game.headers["Event"] = "Iceburn Engine vs itself"
game.headers["White"] = game.headers["Black"] = "Iceburn Engine"

node = game

board = chess.Board()
for m in data:
    k = chess.Move.from_uci(m)
    board.push(k)
    node = node.add_variation(k)

game.headers["Result"] = board.result()

print(game)
