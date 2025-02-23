const path = require("path");

module.exports = (_env, _argv) => ({
	entry: "./src/index.ts",
	devtool: "source-map",
	target: "node",
	module: {
		rules: [
			{
				test: /\.ts$/,
				use: [
					{
						loader: "ts-loader",
						options: {
							compilerOptions: {
								declaration: true,
								declarationDir: "./dist/types"
							},
							transpileOnly: false
						}
					}
				]
			},
			{
				test: /\.wasm$/,
				type: "asset/inline"
			}
		]
	},
	optimization: {
		minimize: false
	},
	resolve: {
		extensions: [".ts", ".js"],
		alias: {
			root: __dirname,
			src: path.resolve(__dirname, "src")
		}
	},
	output: {
		clean: true,
		filename: "index.js",
		library: {
			name: "SyncClient",
			type: "umd"
		},
		path: path.resolve(__dirname, "dist")
	}
});
