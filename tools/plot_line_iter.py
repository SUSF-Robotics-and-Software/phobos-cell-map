'''
Plots a line iteration from a line step report

Enable the "debug_iters" feature to produce reports
'''
#%%
import matplotlib.pyplot as plt
import json

def plot(report):
    fig, ax = plt.subplots()

    ax.plot(
        [report[0]['start_parent'][0], report[0]['end_parent'][0]], 
        [report[0]['start_parent'][1], report[0]['end_parent'][1]], 
        'r'
    )

    ax.set_xticks([0, 1, 2, 3, 4, 5])
    ax.set_yticks([0, 1, 2, 3, 4, 5])
    ax.xaxis.grid(True)
    ax.yaxis.grid(True)

    for i, step in enumerate(report):
        ax.plot(
            step['current_map'][0],
            step['current_map'][1],
            'ob'
        )
        if i > 0:
            a = report[i - 1]['current_map']
        else:
            a = report[0]['start_map']

        b = [a + d for a, d in zip(a, step['delta'])]
        ax.plot(
            [a[0], b[0]],
            [a[1], b[1]],
            '-b'
        )

    ax.set_aspect('equal', 'box')

    plt.show()

if __name__ == '__main__':
    
    with open('../line_step_report.json', 'r') as f:
        report = json.load(f)

        if len(report) > 100:
            report = report[0:100]

        plot(report)
# %%
