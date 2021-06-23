'''
Generic CellMap specific helper class/utility
'''

import numpy as np
import json
import typing

class CellMap:
    data: typing.Dict[str, np.ndarray]
    layers: typing.List[str]
    cell_size: np.ndarray
    num_cells: np.ndarray
    cell_boundary_precision: float
    from_parent: np.ndarray
    to_parent: np.ndarray

    @staticmethod
    def load(path):
        '''
        Loads a CellMap from the given path, expects a JSON file.
        '''

        # Read the data
        with open(path, 'r') as f:
            raw = json.load(f)

        cm = CellMap()

        # Load metadata
        cm.layers = raw['layers']
        cm.cell_size = np.array(raw['cell_size'])
        cm.num_cells = np.array(raw['num_cells'])
        cm.cell_boundary_precision = np.array(raw['cell_boundary_precision'])
        cm.from_parent = np.array(raw['from_parent_matrix']).reshape((3, 3))
        cm.to_parent = np.linalg.inv(cm.from_parent)
        cm.data = dict()

        # Load each layer in turn, reshaping as needed
        for layer, data in zip(cm.layers, raw['data']):
            if data['dim'][0] != cm.num_cells[0] or data['dim'][1] != cm.num_cells[1]:
                raise RuntimeError('Data in cell map file is of wrong shape')
            cm.data[layer] = np.array(data['data']).reshape(cm.num_cells)

        return cm


