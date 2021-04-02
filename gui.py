import chess.engine
import chess.pgn

engine = chess.engine.SimpleEngine.popen_uci("./target/debug/iceburn.exe")

game = chess.pgn.Game()
game.headers["Event"] = "Iceburn Engine vs itself"
game.headers["White"] = game.headers["Black"] = "Iceburn Engine"

node = game

board = chess.Board()
while not board.is_game_over():
    result = engine.play(board, chess.engine.Limit(time=0.1))
    board.push(result.move)
    node = node.add_variation(result.move)

game.headers["Result"] = board.result()

engine.quit()

print(game)
