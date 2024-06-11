import pandas as pd
import numpy as np
import numpy as np
import matplotlib.pyplot as plt

# Read the PI Excel file
data = pd.read_csv("PI_1H.csv")
data = data.set_index("Time")
pi_data = data.dropna()


# Read the BTC Excel file
data = pd.read_csv("BTC_price_1H.csv")
data = data.set_index("Time")
btc_data = data.dropna()


# Vectorization
pi_data_High = pi_data["High"].to_numpy()
pi_data_Low = pi_data["Low"].to_numpy()
BTC_High = btc_data["High"].to_numpy()
BTC_Low = btc_data["Low"].to_numpy()
BTC_Open = btc_data["Open"].to_numpy()
BTC_Close = btc_data["Close"].to_numpy()


def pi_1h_strategy(btc_data, pi_data, period, bb_multiplier):
    # Vectorization
    pi_high_upper = (
        pi_data["High"].rolling(window=period).mean()
        + bb_multiplier * pi_data["High"].rolling(window=period).std()
    )
    pi_low_lower = (
        pi_data["Low"].rolling(window=period).mean()
        - bb_multiplier * pi_data["Low"].rolling(window=period).std()
    )

    pi_high_upper = pi_high_upper.to_numpy()
    pi_low_lower = pi_low_lower.to_numpy()

    # Initialize signal, position, and P&L columns
    BTC_Entry_Signal = np.zeros((len(btc_data)))
    BTC_Signal_with_exits = np.zeros((len(btc_data)))
    BTC_Position = np.zeros((len(btc_data)))
    BTC_Pnl = np.zeros((len(btc_data)))
    BTC_mid_line = np.zeros((len(btc_data)))

    BTC_mid_line = (pi_high_upper + pi_low_lower) / 2

    for i in range(2, len(pi_data)):
        if (
            pi_data_High[i - 2] < pi_high_upper[i - 2]
            and pi_data_High[i - 1] > pi_high_upper[i - 1]
            and BTC_Position[i - 1] == 0
        ):
            BTC_Entry_Signal[i] = -1
            BTC_Position[i] = BTC_Position[i - 1] - 1
        elif (
            pi_data_Low[i - 2]
            > pi_low_lower[i - 2]  # Pi low crossbelow the pi_low lower band
            and pi_data_Low[i - 1] < pi_low_lower[i - 1]
            and BTC_Position[i - 1] == 0  # No position
        ):
            BTC_Entry_Signal[i] = 1  # Long signal , Open long position
            BTC_Position[i] = BTC_Position[i - 1] + 1

        else:
            BTC_Entry_Signal[i] = 0  # No signal
            BTC_Position[i] = BTC_Position[i - 1]

        # Exit conditions
        if (
            BTC_Position[i - 1] == 1
            and pi_data_Low[i - 2] < BTC_mid_line[i - 2]
            and pi_data_Low[i - 1] > BTC_mid_line[i - 1]
        ):
            BTC_Position[i] = 0  # Flat Long signal

        elif (
            BTC_Position[i - 1] == -1
            and pi_data_High[i - 2] > BTC_mid_line[i - 2]
            and pi_data_High[i - 1] < BTC_mid_line[i - 1]
        ):
            BTC_Position[i] = 0  # Flat Short signal

        # Signal with exits
        if BTC_Entry_Signal[i] == -1:
            BTC_Signal_with_exits[i] = BTC_Entry_Signal[i]
        elif BTC_Entry_Signal[i] == 1:
            BTC_Signal_with_exits[i] = BTC_Entry_Signal[i]
        elif (
            BTC_Entry_Signal[i] == 0
            and BTC_Position[i - 1] == -1
            and BTC_Position[i] == 0  # Flat short pos
        ):
            BTC_Signal_with_exits[i] = 1
        elif (
            BTC_Entry_Signal[i] == 0
            and BTC_Position[i - 1] == 1
            and BTC_Position[i] == 0  # Flat long pos
        ):
            BTC_Signal_with_exits[i] = -1
        else:
            BTC_Signal_with_exits[i] = 0

        # Calculate P&L
        if BTC_Position[i] == -1:
            entry_price = BTC_Open[i]
            exit_price = BTC_Close[i]
            BTC_Pnl[i] = -((exit_price / entry_price) - 1)
        elif BTC_Position[i] == 1:
            entry_price = BTC_Open[i]
            exit_price = BTC_Close[i]
            BTC_Pnl[i] = (exit_price / entry_price) - 1

        # Transaction cost
        if BTC_Signal_with_exits[i] == 1 or BTC_Signal_with_exits[i] == -1:
            BTC_Pnl[i] = BTC_Pnl[i] - (0.06 / 100)

    # Calculate cumulative P&L
    BTC_CUM_PNL = np.cumsum(BTC_Pnl)

    return (
        BTC_Open,
        BTC_High,
        BTC_Low,
        BTC_Close,
        BTC_Entry_Signal,
        BTC_Signal_with_exits,
        BTC_Position,
        BTC_Pnl,
        BTC_mid_line,
        BTC_CUM_PNL,
    )


if __name__ == "__main__":
    res = pi_1h_strategy(btc_data, pi_data, 100, 2)
    BTC_CUM_PNL = res[9]
    plt.plot(BTC_CUM_PNL)
    plt.show()
