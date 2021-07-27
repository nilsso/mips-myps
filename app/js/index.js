import ReactDOM from "react-dom";
import Root from "./react/Root";

import("../pkg/index.js")
    .then((acmWasm) => {
        ReactDOM.render(
            React.createElement(Root, { acmWasm }),
            document.getElementById("root")
        );
    })
    .catch(console.error);
