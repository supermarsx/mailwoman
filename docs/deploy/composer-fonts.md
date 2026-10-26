# Composer font assets

The composer catalogue separates locally bundled fonts from common system-font
fallbacks. It does not embed fonts in outgoing messages: recipients may display
another installed font from the serialized stack.

Six additional Google Fonts families are packaged locally through Fontsource:
Source Sans 3 and Source Serif 4 (5.3.1), Lora, Montserrat, Nunito Sans and
Merriweather (5.3.0). Each package declares OFL-1.1 and includes its license.
The exact package licenses ship in `/font-licenses/` with the SPA.
Normal and italic variable weight faces are imported. Vite copies their WOFF2
subsets into the build; no Google Fonts or other font CDN is contacted at runtime.

Existing Jost, Inter, Roboto, Open Sans, Newsreader and JetBrains Mono remain
available. Arial, Verdana, Tahoma, Trebuchet MS, Georgia, Times New Roman and
Courier New are system choices only; Mailwoman does not ship their proprietary
font files.

Primary package documentation: https://fontsource.org/docs/getting-started/install
