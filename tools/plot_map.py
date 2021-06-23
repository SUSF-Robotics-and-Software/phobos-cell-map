'''
Plots a `_map.json` file.
'''

import matplotlib.pyplot as plt
from matplotlib.collections import QuadMesh
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

    # Setup the axis grid
    if parent_relative:
        # Get the meshgrids of cell indices, noting we're doing homogeneous
        x_grid, y_grid, z_grid = np.meshgrid(
            range(map.num_cells[0]), 
            range(map.num_cells[1]), 
            [1.0]
        )

        # Rearrange to a vector of 3D homogenous points, so we can multiply it
        # by our `to_parent` matrix
        positions = np.array((x_grid, y_grid, z_grid))
        
        # Rotate everything
        positions = np.einsum('ij,jabc->iabc', map.to_parent, positions)

        # Reshape back into our meshgrid
        x_grid, y_grid, z_grid = positions

        # Get back our 2D grids
        x_grid[:, :, 0]
        y_grid[:, :, 0]

        # TODO: don't plot every point, just join up lines to points on the
        # edge of the map :(

        quad_mesh = QuadMesh(map.num_cells[0] - 1, map.num_cells[1] - 1, np.array([x_grid, y_grid]))

        ax.add_collection(quad_mesh)
    else:
        x_ticks = range(map.num_cells[0])
        y_ticks = range(map.num_cells[1])
        ax.set_xticks(x_ticks)
        ax.set_yticks(y_ticks)

    if fig is not None:
        fig.show()

if __name__ == '__main__':
    parser = argparse.ArgumentParser(description='Plots a map JSON file')
    parser.add_argument('map', metavar='MAP', type=str, nargs=1, help='path to the map file to plot')
    
    args = parser.parse_args()

    # load the map
    map = CellMap.load(args.map[0])

    plot_map(map, name=args.map[0])

