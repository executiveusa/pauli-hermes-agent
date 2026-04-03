/**
 * Generate CSS Variables from Design Tokens
 * 
 * This script reads design-system/tokens/*.json files and generates
 * CSS custom properties (variables) for use in stylesheets.
 */

const fs = require('fs');
const path = require('path');

// Read token files
function readTokens() {
  const tokensDir = path.join(__dirname, '..', 'tokens');
  const tokenFiles = fs.readdirSync(tokensDir).filter(file => file.endsWith('.json') && file !== 'README.md');
  
  const tokens = {};
  
  for (const file of tokenFiles) {
    const filePath = path.join(tokensDir, file);
    try {
      const content = fs.readFileSync(filePath, 'utf-8');
      const tokenName = file.replace('.json', '');
      tokens[tokenName] = JSON.parse(content);
    } catch (error) {
      console.warn(`Warning: Could not read ${file}: ${error.message}`);
    }
  }
  
  return tokens;
}

// Generate CSS variables from color tokens
function generateColorCSS(colors) {
  let css = ':root {\n';
  css += '  /* Colors */\n';
  
  Object.entries(colors.tokens || colors).forEach(([key, value]) => {
    const cssVar = key.replace(/([A-Z])/g, '-$1').toLowerCase();
    css += `  --color-${cssVar}: ${value};\n`;
  });
  
  return css;
}

// Generate CSS variables from typography tokens
function generateTypographyCSS(typography) {
  let css = '\n  /* Typography */\n';
  
  // Font families
  if (typography.fontFamilies) {
    Object.entries(typography.fontFamilies).forEach(([key, value]) => {
      const cssVar = key.replace(/([A-Z])/g, '-$1').toLowerCase();
      const fontFamily = Array.isArray(value) ? value.join(', ') : value;
      css += `  --font-family-${cssVar}: ${fontFamily};\n`;
    });
  }
  
  // Type scale
  if (typography.typeScale) {
    Object.entries(typography.typeScale).forEach(([scaleKey, scale]) => {
      const cssVar = scaleKey.replace(/([A-Z])/g, '-$1').toLowerCase();
      
      // Desktop, tablet, mobile variants
      if (scale.desktop) {
        css += `  --font-size-${cssVar}-desktop: ${scale.desktop};\n`;
      }
      if (scale.tablet) {
        css += `  --font-size-${cssVar}-tablet: ${scale.tablet};\n`;
      }
      if (scale.mobile) {
        css += `  --font-size-${cssVar}-mobile: ${scale.mobile};\n`;
      }
    });
  }
  
  // Font weights
  if (typography.fontWeights) {
    Object.entries(typography.fontWeights).forEach(([key, value]) => {
      const cssVar = key.replace(/([A-Z])/g, '-$1').toLowerCase();
      css += `  --font-weight-${cssVar}: ${value};\n`;
    });
  }
  
  // Line heights
  if (typography.lineHeights) {
    Object.entries(typography.lineHeights).forEach(([key, value]) => {
      const cssVar = key.replace(/([A-Z])/g, '-$1').toLowerCase();
      css += `  --line-height-${cssVar}: ${value};\n`;
    });
  }
  
  return css;
}

// Generate CSS variables from spacing tokens
function generateSpacingCSS(spacing) {
  let css = '\n  /* Spacing */\n';
  
  Object.entries(spacing.scale || spacing).forEach(([key, value]) => {
    const cssVar = key.replace(/([A-Z])/g, '-$1').toLowerCase();
    css += `  --spacing-${cssVar}: ${value};\n`;
  });
  
  return css;
}

// Generate CSS variables from border radius tokens
function generateBorderCSS(borders) {
  let css = '\n  /* Border Radius */\n';
  
  Object.entries(borders.scale || borders).forEach(([key, value]) => {
    const cssVar = key.replace(/([A-Z])/g, '-$1').toLowerCase();
    css += `  --border-radius-${cssVar}: ${value};\n`;
  });
  
  return css;
}

// Generate CSS variables from shadow tokens
function generateShadowCSS(shadows) {
  let css = '\n  /* Shadows */\n';
  
  Object.entries(shadows.levels || shadows).forEach(([key, value]) => {
    const cssVar = key.replace(/([A-Z])/g, '-$1').toLowerCase();
    const shadowValue = typeof value === 'object' ? value.value : value;
    css += `  --shadow-${cssVar}: ${shadowValue};\n`;
  });
  
  return css;
}

// Generate CSS variables from animation tokens
function generateAnimationCSS(animations) {
  let css = '\n  /* Animations */\n';
  
  // Durations
  if (animations.durations) {
    Object.entries(animations.durations).forEach(([key, value]) => {
      const cssVar = key.replace(/([A-Z])/g, '-$1').toLowerCase();
      css += `  --animation-duration-${cssVar}: ${value};\n`;
    });
  }
  
  // Easing
  if (animations.easing) {
    Object.entries(animations.easing).forEach(([key, value]) => {
      const cssVar = key.replace(/([A-Z])/g, '-$1').toLowerCase();
      css += `  --animation-easing-${cssVar}: ${value};\n`;
    });
  }
  
  return css;
}

// Generate complete CSS file
function generateCSS() {
  const tokens = readTokens();
  let css = '/**\n * Design System CSS Variables\n * Generated from design-system/tokens/*.json\n * DO NOT EDIT MANUALLY - Use generate:css script\n */\n\n';
  
  css += ':root {\n';
  
  if (tokens.colors) {
    css += generateColorCSS(tokens.colors);
  }
  
  if (tokens.typography) {
    css += generateTypographyCSS(tokens.typography);
  }
  
  if (tokens.spacing) {
    css += generateSpacingCSS(tokens.spacing);
  }
  
  if (tokens.borders) {
    css += generateBorderCSS(tokens.borders);
  }
  
  if (tokens.shadows) {
    css += generateShadowCSS(tokens.shadows);
  }
  
  if (tokens.animations) {
    css += generateAnimationCSS(tokens.animations);
  }
  
  css += '}\n\n';
  
  // Add responsive breakpoint variables
  css += '/* Responsive Breakpoints */\n';
  css += ':root {\n';
  css += '  --breakpoint-sm: 640px;\n';
  css += '  --breakpoint-md: 768px;\n';
  css += '  --breakpoint-lg: 1024px;\n';
  css += '  --breakpoint-xl: 1280px;\n';
  css += '  --breakpoint-2xl: 1536px;\n';
  css += '}\n\n';
  
  // Add reduced motion support
  css += '/* Accessibility: Reduced Motion */\n';
  css += '@media (prefers-reduced-motion: reduce) {\n';
  css += '  :root {\n';
  css += '    --animation-duration-fast: 0ms;\n';
  css += '    --animation-duration-normal: 0ms;\n';
  css += '    --animation-duration-slow: 0ms;\n';
  css += '    --animation-duration-page: 0ms;\n';
  css += '  }\n';
  css += '}\n';
  
  return css;
}

// Write CSS file
function writeCSS() {
  const css = generateCSS();
  const outputPath = path.join(__dirname, '..', 'css', 'variables.css');
  
  // Create css directory if it doesn't exist
  try {
    fs.mkdirSync(path.join(__dirname, '..', 'css'), { recursive: true });
  } catch (error) {
    // Directory already exists
  }
  
  fs.writeFileSync(outputPath, css, 'utf-8');
  console.log(`✅ CSS variables generated: ${outputPath}`);
}

// Run generation
writeCSS();