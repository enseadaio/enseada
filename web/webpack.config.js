const MiniCssExtractPlugin = require('mini-css-extract-plugin')
const purgecss = require('@fullhuman/postcss-purgecss')({
  content: [
    './app/js/**/*.js',
    './app/scss/**/*.scss',
    './templates/**/*.html'
  ],
  defaultExtractor: content => content.match(/[\w-/:]+(?<!:)/g) || []
})

const path = require('path')

module.exports = (env, { mode }) => {
  return {
    entry: './app/js/app.js',
    output: {
      filename: 'app.js',
      path: path.resolve(__dirname, 'static')
    },
    module: {
      rules: [
        {
          test: /\.s[ac]ss$/i,
          use: [
            MiniCssExtractPlugin.loader,
            'css-loader',
            {
              loader: 'sass-loader',
              options: {
                // Prefer `dart-sass`
                implementation: require('sass')
              }
            },
            {
              loader: 'postcss-loader',
              options: {
                ident: 'postcss',
                plugins: [
                  require('tailwindcss')
                ].concat(mode === 'production'
                  ? [purgecss]
                  : [])
              }
            }
          ]
        }
      ]
    },
    plugins: [
      new MiniCssExtractPlugin({
        filename: '[name].css',
        chunkFilename: '[id].css'
      })
    ]
  }
}