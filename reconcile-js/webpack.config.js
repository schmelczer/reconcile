const path = require('path');

const common = {
  optimization: {
    // the consuming project should take care of minification
    minimize: false,
  },
  resolve: {
    extensions: ['.ts', '.js'],
    alias: {
      root: __dirname,
      src: path.resolve(__dirname, 'src'),
    },
  },
  performance: {
    hints: false,
  },
  module: {
    rules: [
      {
        test: /\.ts$/,
        use: ['ts-loader'],
      },
      {
        test: /\.wasm$/,
        type: 'asset/inline',
        generator: {
          dataUrl: (content) => content.toString('base64'),
        },
      },
    ],
  },
};

module.exports = [
  // Web build: real WebAssembly, instantiated synchronously from inlined base64.
  {
    ...common,
    target: 'web',
    entry: './src/index.ts',
    output: {
      path: path.resolve(__dirname, 'dist'),
      filename: 'reconcile.web.js',
      library: {
        name: 'reconcile',
        type: 'umd',
      },
      globalObject: 'this',
    },
  },

  // Node build: real WebAssembly.
  {
    ...common,
    target: 'node',
    entry: './src/index.ts',
    output: {
      path: path.resolve(__dirname, 'dist'),
      filename: 'reconcile.node.js',
      libraryTarget: 'commonjs2',
    },
  },

  // React Native build: wasm2js (pure JS), for Hermes which has no
  // `WebAssembly` global. Sources come from `pkg-rn/`
  {
    ...common,
    target: 'web',
    entry: './src/index.rn.ts',
    output: {
      path: path.resolve(__dirname, 'dist'),
      filename: 'reconcile.rn.js',
      library: {
        name: 'reconcile',
        type: 'umd',
      },
      globalObject: 'this',
    },
  },
];
