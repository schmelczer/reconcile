const path = require('path');
const { merge } = require('webpack-merge');

const common = {
  entry: './src/index.ts',
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
  experiments: {
    asyncWebAssembly: true,
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
  merge(common, {
    target: 'web',
    output: {
      path: path.resolve(__dirname, 'dist'),
      filename: 'reconcile.web.js',
      library: {
        name: 'reconcile',
        type: 'umd',
      },
      globalObject: 'this',
    },
  }),
  merge(common, {
    target: 'node',
    output: {
      path: path.resolve(__dirname, 'dist'),
      filename: 'reconcile.node.js',
      libraryTarget: 'commonjs2',
    },
  }),
];
