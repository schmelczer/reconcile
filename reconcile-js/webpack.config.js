const path = require("path");


module.exports = {
    entry: "./src/index.ts",
    module: {
        rules: [
            {
                test: /\.ts$/,
                use: ["ts-loader"]
            },
            {
                test: /\.wasm$/,
                type: "asset/inline"
            }
        ]
    },
    optimization: {
        // the consuming project should take care of minification
        minimize: false
    },
    resolve: {
        extensions: [".ts"],
        alias: {
            root: __dirname,
            src: path.resolve(__dirname, "src")
        }
    },
    output: {
        path: path.resolve(__dirname, "dist"),
        filename: "index.js",
        libraryTarget: "module"
    },
    experiments: {
        outputModule: true
    },
};
