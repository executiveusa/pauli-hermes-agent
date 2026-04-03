/**
 * Generate Tailwind Config from Design Tokens
 * 
 * This script reads design-system/tokens/*.json files and generates
 * a Tailwind CSS configuration that extends the default theme.
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

// Generate Tailwind colors theme
function generateColors(colors) {
  const colorMap = {};
  
  Object.entries(colors.tokens || colors).forEach(([key, value]) => {
    // Convert camelCase to kebab-case for Tailwind
    const tailwindKey = key.replace(/([A-Z])/g, '-$1').toLowerCase();
    colorMap[tailwindKey] = value;
  });
  
  return colorMap;
}

// Generate Tailwind fontFamily theme
function generateFontFamilies(typography) {
  const fontFamily = {};
  
  if (typography.fontFamilies) {
    Object.entries(typography.fontFamilies).forEach(([key, value]) => {
      const tailwindKey = key.replace(/([A-Z])/g, '-$1').toLowerCase();
      // Tailwind expects arrays for font families
      fontFamily[tailwindKey] = Array.isArray(value) ? value : [value];
    });
  }
  
  return fontFamily;
}

// Generate Tailwind fontSize theme
function generateFontSizes(typography) {
  const fontSize = {};
  
  if (typography.typeScale) {
    Object.entries(typography.typeScale).forEach(([scaleKey, scale]) => {
      const tailwindKey = scaleKey.replace(/([A-Z])/g, '-$1').toLowerCase();
      
      // Use desktop values as base, with responsive variants
      if (scale.desktop) {
        const lineHeight = typography.lineHeights?.normal || 1.5;
        fontSize[tailwindKey] = [
          `${scale.desktop}`,
          { lineHeight: `${lineHeight}` }
        ];
      }
    });
  }
  
  return fontSize;
}

// Generate Tailwind fontWeight theme
function generateFontWeights(typography) {
  const fontWeight = {};
  
  if (typography.fontWeights) {
    Object.entries(typography.fontWeights).forEach(([key, value]) => {
      const tailwindKey = key.replace(/([A-Z])/g, '-$1').toLowerCase();
      fontWeight[tailwindKey] = value;
    });
  }
  
  return fontWeight;
}

// Generate Tailwind lineHeight theme
function generateLineHeights(typography) {
  const lineHeight = {};
  
  if (typography.lineHeights) {
    Object.entries(typography.lineHeights).forEach(([key, value]) => {
      const tailwindKey = key.replace(/([A-Z])/g, '-$1').toLowerCase();
      lineHeight[tailwindKey] = value;
    });
  }
  
  return lineHeight;
}

// Generate Tailwind spacing theme
function generateSpacing(spacing) {
  const spacing = {};
  
  Object.entries(spacing.scale || spacing).forEach(([key, value]) => {
    spacing[key] = value;
  });
  
  return spacing;
}

// Generate Tailwind borderRadius theme
function generateBorderRadius(borders) {
  const borderRadius = {};
  
  Object.entries(borders.scale || borders).forEach(([key, value]) => {
    const tailwindKey = key.replace(/([A-Z])/g, '-$1').toLowerCase();
    borderRadius[tailwindKey] = value;
  });
  
  return borderRadius;
}

// Generate Tailwind boxShadow theme
function generateBoxShadow(shadows) {
  const boxShadow = {};
  
  Object.entries(shadows.levels || shadows).forEach(([key, value]) => {
    const tailwindKey = key.replace(/([A-Z])/g, '-$1').toLowerCase();
    const shadowValue = typeof value === 'object' ? value.value : value;
    boxShadow[tailwindKey] = shadowValue;
  });
  
  return boxShadow;
}

// Generate Tailwind transitionDuration theme
function generateTransitionDuration(animations) {
  const transitionDuration = {};
  
  if (animations.durations) {
    Object.entries(animations.durations).forEach(([key, value]) => {
      const tailwindKey = key.replace(/([A-Z])/g, '-$1').toLowerCase();
      transitionDuration[tailwindKey] = value;
    });
  }
  
  return transitionDuration;
}

// Generate Tailwind transitionTimingFunction theme
function generateTransitionTimingFunction(animations) {
  const transitionTimingFunction = {};
  
  if (animations.easing) {
    Object.entries(animations.easing).forEach(([key, value]) => {
      const tailwindKey = key.replace(/([A-Z])/g, '-$1').toLowerCase();
      transitionTimingFunction[tailwindKey] = value;
    });
  }
  
  return transitionTimingFunction;
}

// Generate complete Tailwind config
function generateTailwindConfig() {
  const tokens = readTokens();
  
  const config = {
    content: [
      './src/**/*.{js,ts,jsx,tsx}',
      './components/**/*.{js,ts,jsx,tsx}',
      './app/**/*.{js,ts,jsx,tsx}',
      './pages/**/*.{js,ts,jsx,tsx}',
    ],
    darkMode: 'class',
    theme: {
      extend: {}
    }
  };
  
  if (tokens.colors) {
    config.theme.extend.colors = generateColors(tokens.colors);
  }
  
  if (tokens.typography) {
    config.theme.extend.fontFamily = generateFontFamilies(tokens.typography);
    config.theme.extend.fontSize = generateFontSizes(tokens.typography);
    config.theme.extend.fontWeight = generateFontWeights(tokens.typography);
    config.theme.extend.lineHeight = generateLineHeights(tokens.typography);
  }
  
  if (tokens.spacing) {
    config.theme.extend.spacing = generateSpacing(tokens.spacing);
  }
  
  if (tokens.borders) {
    config.theme.extend.borderRadius = generateBorderRadius(tokens.borders);
  }
  
  if (tokens.shadows) {
    config.theme.extend.boxShadow = generateBoxShadow(tokens.shadows);
  }
  
  if (tokens.animations) {
    config.theme.extend.transitionDuration = generateTransitionDuration(tokens.animations);
    config.theme.extend.transitionTimingFunction = generateTransitionTimingFunction(tokens.animations);
  }
  
  return config;
}

// Write Tailwind config file
function writeTailwindConfig() {
  const config = generateTailwindConfig();
  const jsContent = `/** @type {import('tailwindcss').Config} */
module.exports = ${JSON.stringify(config, null, 2)};`;
  
  const outputPath = path.join(__dirname, '..', 'tailwind.config.js');
  
  fs.writeFileSync(outputPath, jsContent, 'utf-8');
  console.log(`✅ Tailwind config generated: ${outputPath}`);
}

// Run generation
writeTailwindConfig();