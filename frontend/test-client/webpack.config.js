const path = require("path");
const webpack = require("webpack");

module.exports = {
	entry: "./src/cli.ts",
	target: "node",
	mode: "production",
	optimization: {
		minimize: false
	},
	module: {
		rules: [
			{
				test: /\.ts$/,
				use: "ts-loader"
			}
		]
	},
	resolve: {
		extensions: [".ts", ".js"]
	},
	output: {
		globalObject: "this",
		filename: "cli.js",
		path: path.resolve(__dirname, "dist")
	},
	plugins: [
		new webpack.BannerPlugin({ banner: "#!/usr/bin/env node", raw: true })
	]
};
