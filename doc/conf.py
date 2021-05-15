#!/usr/bin/env python3
# -*- coding: utf-8 -*-
#
# parts of this were originally generated by sphinx-quickstart on Thu Aug 31 17:31:36 2017.

import os.path
import sys

sys.path.append(os.path.abspath('..'))

project = 'rerobots CLI'
copyright = '2021 rerobots, Inc'
author = 'rerobots, Inc.'
html_logo = '_static/logo.svg'

version = ''
release = ''

language = None

extensions = ['sphinx.ext.autodoc']

autoclass_content = 'init'

source_suffix = '.rst'
master_doc = 'index'
exclude_patterns = []

templates_path = ['_templates']
pygments_style = 'sphinx'

# read more about customization of this style at
# http://alabaster.readthedocs.io/en/stable/customization.html
html_theme = 'alabaster'
html_sidebars = {
}
html_theme_options = {
    'show_powered_by': 'false'
}


# Prepare to build on hosts of https://readthedocs.org/
import os
if os.environ.get('READTHEDOCS', 'False') == 'True':
    import subprocess
    subprocess.check_call('./get-deps.sh')
