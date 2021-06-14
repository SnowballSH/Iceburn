import chess.engine
import chess.pgn

engine1 = chess.engine.SimpleEngine.popen_uci("./target/release/iceburn.exe")
engine2 = chess.engine.SimpleEngine.popen_uci("./iceburn_a.exe")

FEN = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"

game = chess.pgn.Game()
game.headers["Event"] = "Iceburn Engine vs itself"
game.headers["White"] = "Iceburn Engine"
game.headers["Black"] = "Iceburn Engine (old)"
game.headers["FEN"] = FEN

node = game

print("calculating...")

board = chess.Board(fen=FEN)
game.from_board(board)
while not board.is_game_over():
    result = engine1.play(board, chess.engine.Limit(white_clock=3, black_clock=3, white_inc=1, black_inc=1))
    board.push(result.move)
    print(board)
    print()
    node = node.add_variation(result.move)

    if board.is_game_over():
        break

    result = engine2.play(board, chess.engine.Limit(white_clock=3, black_clock=3, white_inc=1, black_inc=1))
    board.push(result.move)
    node = node.add_variation(result.move)

    # board.push_san((input().strip()))

    print(board)
    print()

game.headers["Result"] = board.result()

engine1.quit()
engine2.quit()

print(game)
