pub const STANDALONE_HTML_HEADER: &str = "
<!DOCTYPE html>
<html>
  <body>
    <style type=\"text/css\">
      body { font-family: \"Fira Sans\", arial; }
      td {
        padding: 1em;
      }
      table, td, tr {
        border: 2px solid;
        border-spacing: 0px;
      }
      .ingredient {
        background-color: #ddd;
      }
      .done {
        background-color: #555;
      }
      .amount { color: #555; }
      .seasonings { color: #333; }
    </style>
";

pub const STANDALONE_HTML_FOOTER: &str = "
  </body>
</html>
";
