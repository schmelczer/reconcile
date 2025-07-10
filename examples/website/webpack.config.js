const path = require('path');
const HtmlWebpackPlugin = require('html-webpack-plugin');
const TerserPlugin = require('terser-webpack-plugin');
const InlineSourceWebpackPlugin = require('inline-source-webpack-plugin');
const MiniCssExtractPlugin = require('mini-css-extract-plugin');

module.exports = (_env, argv) => ({
  devtool: argv.mode === 'development' ? 'inline-source-map' : false,
  entry: {
    index: './src/index.ts',
  },
  devServer: {
    allowedHosts: 'all',
  },
  watchOptions: {
    ignored: '**/node_modules',
  },
  optimization: {
    minimizer: [
      new TerserPlugin({
        terserOptions: {
          module: true,
        },
      }),
    ],
  },
  performance: {
    assetFilter: (f) => !/\.(webm|mp4|pdf)$/.test(f),
    maxEntrypointSize: 100000,
    maxAssetSize: 512000,
  },
  plugins: [
    new HtmlWebpackPlugin({
      template: './src/index.html',
    }),
    new MiniCssExtractPlugin(),
    argv.mode === 'production'
      ? new InlineSourceWebpackPlugin({
          compress: true,
        })
      : null,
  ].filter(Boolean),
  module: {
    rules: [
      {
        test: /\.svg$/i,
        use: 'svg-inline-loader',
      },
      {
        test: /\.scss$/i,
        use: [
          MiniCssExtractPlugin.loader,
          'css-loader',
          'resolve-url-loader',
          {
            loader: 'sass-loader',
            options: {
              sourceMap: true, // required by resolve-url-loader
            },
          },
        ],
      },
      {
        test: /\.ts$/,
        use: 'ts-loader',
      },
    ],
  },
  resolve: {
    extensions: [
      '.ts',
      '.js', // required for development
    ],
  },
  output: {
    clean: true,
    filename: '[name].js',
    path: path.resolve(__dirname, 'dist'),
    publicPath: '',
  },
});
