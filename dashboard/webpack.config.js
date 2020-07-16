const path = require('path')
const WasmPackPlugin = require('@wasm-tool/wasm-pack-plugin')
const CopyWebpackPlugin = require('copy-webpack-plugin')
const MiniCssExtractPlugin = require('mini-css-extract-plugin')

const distPath = path.resolve(__dirname, 'dist')
module.exports = (env, argv) => {
  return {
    devServer: {
      contentBase: distPath,
      compress: argv.mode === 'production',
      historyApiFallback: true,
      port: 8000
    },
    entry: './bootstrap.js',
    output: {
      path: distPath,
      filename: 'enseada.js',
      webassemblyModuleFilename: 'enseada.wasm',
      publicPath: '/static/'
    },
    module: {
      rules: [
        {
          test: /\.s[ac]ss$/i,
          use: [
            'style-loader',
            MiniCssExtractPlugin.loader,
            'css-loader',
            'sass-loader'
          ]
        }
      ]
    },
    plugins: [
      new CopyWebpackPlugin([
        { from: './static', to: distPath }
      ]),
      new WasmPackPlugin({
        crateDirectory: '.',
        extraArgs: '--no-typescript'
      }),
      new MiniCssExtractPlugin({
        filename: 'enseada.css',
        chunkFilename: '[id].css'
      })
    ],
    watch: argv.mode !== 'production'
  }
}