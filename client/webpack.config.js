const path = require('path');
const CopyPlugin = require('copy-webpack-plugin');
const WasmPackPlugin = require('@wasm-tool/wasm-pack-plugin');

const dist = path.resolve(__dirname, 'dist');

module.exports = {
  mode: 'production',
  entry: './js/index.ts',
  output: {
    path: dist,
    filename: 'index.js',
  },
  devServer: {
    compress: true,
    port: 3000,
  },
  plugins: [
    new CopyPlugin({ patterns: [path.resolve(__dirname, 'static')] }),
    new WasmPackPlugin({ crateDirectory: __dirname }),
  ],
  module: {
    rules: [
      {
        test: /\.tsx?$/,
        use: 'ts-loader',
        exclude: /node_modules/,
      },
    ],
  },
  resolve: {
    extensions: ['.tsx', '.ts', '.js'],
  },
  experiments: {
    syncWebAssembly: true,
  },
};
