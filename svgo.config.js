module.exports = {
  multipass: true,
  plugins: [
    "preset-default",
    "removeDimensions",
    {
      name: "removeAttrs",
      params: {
        attrs: ["data-name"],
      },
    },
    {
      name: "sortAttrs",
      params: {
        xmlnsOrder: "alphabetical",
      },
    },
  ],
};
