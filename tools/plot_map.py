'''
Plots a `_map.json` file.
'''

import matplotlib.pyplot as plt
from matplotlib.collections import LineCollection
import numpy as np
import argparse
from cell_map import CellMap

def plot_map(map: CellMap, name = None, ax = None, parent_relative = True):
    '''
    Plots the given CellMap

    Arguments:
        map: The CellMap to plot
        name: The name to place in the title
        ax: The axis to plot onto, or None if we should create a new figure
        parent_relative: True if the map should be plotted relative to the parent
    '''
    
    if ax is None:
        fig, ax = plt.subplots()
    else:
        fig = None

    # Map-relative origin and axes directions
    origin = np.array([0.0, 0.0])
    x_dir = np.array([1.0, 0.0])
    y_dir = np.array([0.0, 1.0])

    # Map-relative limits
    x_lims = [-0.5, map.num_cells[0] + 0.5]
    y_lims = [-0.5, map.num_cells[1] + 0.5]

    # Setup the axis grid
    if parent_relative:
        # Get the grid
        line_collection = _get_parent_rel_grid(map)

        # Plot lines
        ax.add_collection(line_collection)

        # Update map origin
        origin = map.transform_to_parent(origin.reshape((1, 2))).reshape((2,))
        x_dir = map.transform_to_parent(x_dir.reshape((1, 2))).reshape((2,)) - origin
        y_dir = map.transform_to_parent(y_dir.reshape((1, 2))).reshape((2,)) - origin

        plot_bounds = np.max([map.cell_size[0] * 0.5, map.cell_size[1] * 0.5])

        # Update limits
        x_lims = [np.min(map.extents[:,0]) - plot_bounds, np.max(map.extents[:,0]) + plot_bounds]
        y_lims = [np.min(map.extents[:,1]) - plot_bounds, np.max(map.extents[:,1]) + plot_bounds]

    else:
        # Include the end line in the ticks
        x_ticks = range(map.num_cells[0] + 1)
        y_ticks = range(map.num_cells[1] + 1)
        ax.set_xticks(x_ticks)
        ax.set_yticks(y_ticks)
        ax.grid(True)

    # Plot origin and directions
    ax.plot(origin[0], origin[1], '.k')
    ax.quiver(*origin, x_dir[0], x_dir[1], color='r', angles='xy', scale_units='xy', scale=1)
    ax.quiver(*origin, y_dir[0], y_dir[1], color='g', angles='xy', scale_units='xy', scale=1)

    # Set limits
    ax.set_xlim(x_lims)
    ax.set_ylim(y_lims)
    ax.set_aspect('equal', 'box')

    if fig is not None:
        fig.show()

def _get_parent_rel_grid(map):
    '''
    Gets the parent-relative grid as a matplotlib.collections.LineCollection
    '''

    # Approach:
    #   Get the [x, y] coordinates of all points along the edge of the map,
    #   in map frame coordinates, then transform all of those into parent
    #   frame, and create lines between each opposing pair of points.

    # Get x and y coordinates to plot grid lines on, we will choose to do
    # each cell. This is in the map frame. Do +1 so we get 1 more lines than
    # there are cells.
    x_grids = np.array(range(map.num_cells[0] + 1))
    y_grids = np.array(range(map.num_cells[1] + 1))

    # Four arrays of points, one for each side of the map, must be in
    # homogenous coordinates so we can transform them using the 3x3
    # matrices. Indices of the aisles are [bot, top] for x, [left, right]
    # for y.
    points_x = np.empty((2, len(x_grids), 2))
    points_x[:,:,0] = x_grids
    points_x[0,:,1] = y_grids[0]
    points_x[1,:,1] = y_grids[-1]

    points_y = np.empty((2, len(y_grids), 2))
    points_y[0,:,0] = x_grids[0]
    points_y[1,:,0] = x_grids[-1]
    points_y[:,:,1] = y_grids
    
    # Transform each point into parent frame
    points_x[0] = map.transform_to_parent(points_x[0])
    points_x[1] = map.transform_to_parent(points_x[1])
    points_y[0] = map.transform_to_parent(points_y[0])
    points_y[1] = map.transform_to_parent(points_y[1])

    # Create lines 
    vert_lines = np.hstack((points_x[0], points_x[1]))
    hori_lines = np.hstack((points_y[0], points_y[1]))
    lines = np.concatenate([hori_lines, vert_lines]).reshape(-1, 2, 2)
    return LineCollection(lines, color="gray", linewidths=1)

if __name__ == '__main__':
    parser = argparse.ArgumentParser(description='Plots a map JSON file')
    parser.add_argument('map', metavar='MAP', type=str, nargs=1, help='path to the map file to plot')
    
    args = parser.parse_args()

    # load the map
    map = CellMap.load(args.map[0])

    plot_map(map, name=args.map[0])

