const path = require("path");
const CompressionPlugin = require("compression-webpack-plugin");
const HtmlWebpackPlugin = require("html-webpack-plugin");
const WasmPackPlugin = require("@wasm-tool/wasm-pack-plugin");
const ForkTsCheckerWebpackPlugin = require("fork-ts-checker-webpack-plugin");
const EslintWebpackPlugin = require("eslint-webpack-plugin");
const { GenerateSW } = require("workbox-webpack-plugin");

module.exports = (env, argv) => {
  const appPath = path.resolve(__dirname, "src");
  const outputPath = path.resolve(__dirname, "build/dist");

  const isProduction = argv.mode == "production";
  console.log("Production?", isProduction);

  return {
    mode: argv.mode,

    entry: appPath,

    output: {
      filename: isProduction ? "bundle.[fullhash].js" : "bundle.js",
      path: outputPath,
    },

    resolve: {
      extensions: [".ts", ".tsx", ".js", ".json"],
    },

    experiments: {
      asyncWebAssembly: true,
    },

    module: {
      rules: [
        { test: /\.wasm$/, type: "webassembly/async", exclude: /node_modules/ },
        {
          test: /\.(ts|js)x?$/,
          loader: "babel-loader",
          exclude: /node_modules/,
        },
        {
          test: /\.css$/,
          use: ["style-loader", "css-loader", "postcss-loader"],
        },
      ],
    },

    plugins: [
      new HtmlWebpackPlugin({
        inject: true,
        template: path.join(appPath, "index.html"),
      }),
      new WasmPackPlugin({
        crateDirectory: path.resolve(__dirname, "../portal_adapter"),
        outDir: path.resolve(__dirname, "./build/adapter_pkg"),
        forceMode: "production",
      }),
      new EslintWebpackPlugin({
        exclude: ["node_modules", "build"],
      }),
      new ForkTsCheckerWebpackPlugin(),
      ...(isProduction
        ? [
            new CompressionPlugin({
              algorithm: "brotliCompress",
              test: /\.(js|css|html|svg|wasm)$/,
              threshold: 10240,
              minRatio: 0.8,
            }),
            // This will cache JS/WASM assets for the production build, using a service worker
            //
            // This is especially helpful to start preloading the web worker and WASM files before the simulations are
            // actually run, and to prevent us from having to go to the network for those files every time the
            // simulations are re-run.
            new GenerateSW({
              skipWaiting: true,
              runtimeCaching: [
                // NetworkFirst for index.html to ensure we always get the latest version.
                {
                  urlPattern: /.*index.html$/,
                  handler: "NetworkFirst",
                },
                // For other assets, we can rely on the versioned naming for freshness instead.
                {
                  urlPattern: /.*/,
                  handler: "CacheFirst",
                  options: {
                    cacheName: "mainCache",
                    expiration: {
                      maxAgeSeconds: 24 * 60 * 60, // 1 day
                    },
                  },
                },
              ],
            }),
          ]
        : []),
    ],

    devServer: {
      open: true,
    },
  };
};
