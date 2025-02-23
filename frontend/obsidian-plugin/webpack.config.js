const path = require("path");
const TerserPlugin = require("terser-webpack-plugin");
const MiniCssExtractPlugin = require("mini-css-extract-plugin");
const fs = require("fs-extra");

module.exports = (env, argv) => ({
	devtool: argv.mode === "development" ? "inline-source-map" : false,
	entry: {
		index: "./src/vault-link-plugin.ts"
	},
	watchOptions: {
		ignored: "**/node_modules"
	},
	externals: {
		obsidian: "commonjs obsidian"
	},
	optimization: {
		minimizer: [
			new TerserPlugin({
				terserOptions: {
					module: true
				}
			})
		]
	},
	plugins: [
		new MiniCssExtractPlugin({
			filename: "styles.css"
		}),
		{
			apply: (compiler) => {
				if (argv.mode !== "development") {
					return;
				}

				compiler.hooks.done.tap("Copy Files Plugin", (stats) => {
					const source = path.resolve(__dirname, "dist");
					const destinations = [
						"/mnt/c/Users/Andras/Desktop/test/test/.obsidian/plugins/my-plugin",
						"/mnt/c/Users/Andras/Desktop/test/test2/.obsidian/plugins/my-plugin",
						"/home/andras/obsidian-test/.obsidian/plugins/my-plugin"
					];
					destinations.forEach((destination) => {
						fs.copy(source, destination)
							.then(() =>
								console.log(
									"Files copied successfully after build!"
								)
							)
							.catch((err) =>
								console.error("Error copying files:", err)
							);

						fs.createFile(path.join(destination, ".hotreload"));
					});
				});
			}
		}
	],
	module: {
		rules: [
			{
				test: /\.json$/i,
				type: "asset/resource",
				generator: {
					filename: "[name][ext]"
				}
			},
			{
				test: /\.scss$/i,
				use: [
					MiniCssExtractPlugin.loader,
					"css-loader",
					"resolve-url-loader",
					{
						loader: "sass-loader",
						options: {
							sourceMap: true // required by resolve-url-loader
						}
					}
				]
			},
			{
				test: /\.ts$/,
				use: ["ts-loader"]
			}
		]
	},
	resolve: {
		extensions: [
			".ts",
			".js" // required for development
		],
		alias: {
			root: __dirname,
			src: path.resolve(__dirname, "src")
		},
		fallback: {
			url: require.resolve("url")
		}
	},
	output: {
		clean: true,
		filename: "main.js",
		library: {
			type: "commonjs" // required for Obsidian
		},
		path: path.resolve(__dirname, "dist"),
		publicPath: ""
	}
});
