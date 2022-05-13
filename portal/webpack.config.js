const path = require("path");
const BrotliPlugin = require("brotli-webpack-plugin");
const HtmlWebpackPlugin = require("html-webpack-plugin");
const WasmPackPlugin = require("@wasm-tool/wasm-pack-plugin");
const ForkTsCheckerWebpackPlugin = require("fork-ts-checker-webpack-plugin");
const EslintWebpackPlugin = require("eslint-webpack-plugin");

module.exports = (env, argv) => {
  const appPath = path.resolve(__dirname, "site");
  const outputPath = path.resolve(__dirname, "dist");

  const isProduction = argv.mode == "production";

  return {
    mode: argv.mode,

    entry: appPath,

    output: {
      filename: "bundle.js",
      path: outputPath,
    },

    resolve: {
      extensions: [".ts", ".tsx", ".js", ".json"],
    },

    module: {
      rules: [
        {
          test: /\.(ts|js)x?$/,
          loader: "babel-loader",
          exclude: /node_modules/,
        },
        { test: /\.wasm$/, type: "webassembly/async", exclude: /node_modules/ },
        {
          test: /\.css$/,
          use: ["style-loader", "css-loader", "postcss-loader"],
        },
      ],
    },

    plugins: [
      ...(isProduction
        ? [
            new BrotliPlugin({
              asset: "[path].br[query]",
              test: /\.(js|css|html|svg|wasm)$/,
              threshold: 10240,
              minRatio: 0.8,
            }),
          ]
        : []),
      new HtmlWebpackPlugin({
        inject: true,
        template: path.join(appPath, "index.html"),
      }),
      new WasmPackPlugin({
        crateDirectory: path.resolve(__dirname, "relltee_adapter"),
        outDir: path.resolve(__dirname, "relltee_adapter/pkg"),
      }),
      new EslintWebpackPlugin(),
      new ForkTsCheckerWebpackPlugin(),
    ],

    devServer: {
      open: true,
    },
  };
};
