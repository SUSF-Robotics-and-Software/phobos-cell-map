'''
Plots a line iteration from a line step report

Enable the "debug_iters" feature to produce reports
'''
#%%
import matplotlib.pyplot as plt
import numpy as np
from math import floor, ceil
import json

def plot(report):
    fig, ax = plt.subplots()

    ax.plot(
        [report[0]['start_map'][0], report[0]['end_map'][0]], 
        [report[0]['start_map'][1], report[0]['end_map'][1]], 
        'r'
    )

    ax.set_xticks(np.arange(report[0]['current_map'][0], report[-1]['current_map'][0], 1.0))
    ax.set_yticks(np.arange(report[0]['current_map'][1], report[-1]['current_map'][1], -1.0))
    ax.xaxis.grid(True)
    ax.yaxis.grid(True)

    for i, step in enumerate(report):
        ax.plot(
            step['current_map'][0],
            step['current_map'][1],
            'ob'
        )
        ax.annotate(
            f'{i}',
            (step['current_map'][0], step['current_map'][1])
        )
        if i > 0:
            a = report[i - 1]['current_map']
        else:
            a = report[0]['start_map']

        # b = [a + d for a, d in zip(a, step['delta'])]
        # ax.plot(
        #     [a[0], b[0]],
        #     [a[1], b[1]],
        #     '-b'
        # )

    # ax.set_aspect('equal', 'box')

    plt.show()

if __name__ == '__main__':
    
    with open('../line_step_report.json', 'r') as f:
        report = json.load(f)

        if len(report) > 100:
            report = report[0:100]

        plot(report)
# %%
