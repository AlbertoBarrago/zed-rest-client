; Methods
(method) @keyword

; Headers
(header
  name: (_) @attribute)

; Variables
(variable_declaration
  name: (identifier) @variable)

; Operators
(variable_declaration
  "=" @operator)

; Keywords
(comment
  "@" @keyword
  name: (_) @keyword)

; URLs
(request
  url: (_) @string.special.url)

; HTTP version
(http_version) @constant

; Response status
(status_code) @number
(status_text) @string

; Variable interpolation braces
[
  "{{"
  "}}"
] @punctuation.special

; Header colon
(header
  ":" @punctuation.delimiter)

; External body path
(external_body
  path: (_) @string.special.path)

; Request separator and comments
[
  (comment)
  (request_separator)
] @comment
