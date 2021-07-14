'''
Plots a `_map.json` file.
'''

import matplotlib.pyplot as plt
from matplotlib.collections import LineCollection
import numpy as np
import argparse
from cell_map import CellMap

def plot_map(map: CellMap, name = None, ax = None, parent_relative = True, show_grid=False):
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
        ax.add_artist(_get_parent_rel_grid(map, show_grid=show_grid))

        # Update map origin
        origin = map.transform_to_parent(origin.reshape((1, 2))).reshape((2,))
        x_dir = map.transform_to_parent(x_dir.reshape((1, 2))).reshape((2,)) - origin
        y_dir = map.transform_to_parent(y_dir.reshape((1, 2))).reshape((2,)) - origin

        plot_bounds = np.max([map.cell_size[0] * 0.5, map.cell_size[1] * 0.5])

        # Update limits
        ext_plus_origin_x = np.append(map.extents[:,0], origin[0])
        ext_plus_origin_y = np.append(map.extents[:,1], origin[1])
        x_lims = [np.min(ext_plus_origin_x) - plot_bounds, np.max(ext_plus_origin_x) + plot_bounds]
        y_lims = [np.min(ext_plus_origin_y) - plot_bounds, np.max(ext_plus_origin_y) + plot_bounds]

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

    if name is not None:
        ax.set_title(name)
    else:
        ax.set_title(map.path)

    if fig is not None:
        plt.show()

def _get_parent_rel_grid(map, show_grid=False):
    '''
    Gets the parent-relative grid as a matplotlib.collections.LineCollection
    '''

    # Create mesh grid points by transforming each meshgrid point into the
    # parent frame
    mesh_x, mesh_y = np.meshgrid(
        np.array(range(map.cell_bounds[0][0], map.cell_bounds[0][1] + 1)),
        np.array(range(map.cell_bounds[1][0], map.cell_bounds[1][1] + 1))
    )
    mesh_shape = mesh_x.shape
    mesh_points = np.vstack([mesh_x.ravel(), mesh_y.ravel()]).T
    mesh_points = map.transform_to_parent(mesh_points)
    mesh_x, mesh_y = [mesh_points[:,0].reshape(mesh_shape), mesh_points[:,1].reshape(mesh_shape)]
    
    mesh = plt.pcolormesh(
        mesh_x, mesh_y, map.data[map.layers[0]], 
        shading='flat', 
        edgecolors='grey' if show_grid else None, 
        linewidth=0.1, 
        zorder=-1.0
    )

    return mesh

if __name__ == '__main__':
    parser = argparse.ArgumentParser(description='Plots a map JSON file')
    parser.add_argument('map', metavar='MAP', type=str, nargs=1, help='path to the map file to plot')
    
    args = parser.parse_args()

    # load the map
    map = CellMap.load(args.map[0])

    plot_map(map, name=args.map[0])

