// Copyright 2025 OfficeCli (officecli.ai)
// SPDX-License-Identifier: Apache-2.0

using System.Text.RegularExpressions;
using DocumentFormat.OpenXml;
using DocumentFormat.OpenXml.Packaging;
using DocumentFormat.OpenXml.Presentation;
using OfficeCli.Core;
using Drawing = DocumentFormat.OpenXml.Drawing;

namespace OfficeCli.Handlers;

public partial class PowerPointHandler
{
    private const string Am3dNs = "http://schemas.microsoft.com/office/drawing/2017/model3d";
    private const string Model3dRelType = "http://schemas.microsoft.com/office/2017/06/relationships/model3d";
    // PowerPoint uses "model/gltf.binary" (dot, not dash)
    private const string GlbContentType = "model/gltf.binary";

    private string AddModel3D(string parentPath, int? index, Dictionary<string, string> properties)
    {
        if (!properties.TryGetValue("path", out var modelPath) &&
            !properties.TryGetValue("src", out modelPath))
            throw new ArgumentException("'path' or 'src' property is required for 3dmodel type");

        var slideMatch = Regex.Match(parentPath, @"^/slide\[(\d+)\]$");
        if (!slideMatch.Success)
            throw new ArgumentException("3D models must be added to a slide: /slide[N]");

        var slideIdx = int.Parse(slideMatch.Groups[1].Value);
        var slideParts = GetSlideParts().ToList();
        if (slideIdx < 1 || slideIdx > slideParts.Count)
            throw new ArgumentException($"Slide {slideIdx} not found (total: {slideParts.Count})");

        // Resolve file path
        var fullPath = Path.GetFullPath(modelPath);
        if (!File.Exists(fullPath))
            throw new FileNotFoundException($"3D model file not found: {modelPath}");

        var fileExt = Path.GetExtension(fullPath).ToLowerInvariant();
        if (fileExt != ".glb")
            throw new ArgumentException($"Unsupported 3D model format: {fileExt}. Only .glb (glTF-Binary) is supported.");

        var slidePart = slideParts[slideIdx - 1];
        var shapeTree = GetSlide(slidePart).CommonSlideData?.ShapeTree
            ?? throw new InvalidOperationException("Slide has no shape tree");

        // Parse GLB bounding box for centering
        var glbBounds = ParseGlbBoundingBox(fullPath);

        // Embed .glb file as an extended part
        var modelPart = slidePart.AddExtendedPart(Model3dRelType, GlbContentType, ".glb");
        using (var fs = File.OpenRead(fullPath))
            modelPart.FeedData(fs);
        var modelRelId = slidePart.GetIdOfPart(modelPart);

        // Create fallback placeholder image
        byte[] placeholderPng = GenerateZoomPlaceholderPng();
        var imagePart = slidePart.AddImagePart(ImagePartType.Png);
        using (var ms = new MemoryStream(placeholderPng))
            imagePart.FeedData(ms);
        var imageRelId = slidePart.GetIdOfPart(imagePart);

        // Position and size (default: 10cm x 10cm, centered)
        long cx = 3600000; // ~10cm
        long cy = 3600000;
        if (properties.TryGetValue("width", out var w)) cx = ParseEmu(w);
        if (properties.TryGetValue("height", out var h)) cy = ParseEmu(h);
        var (slideW, slideH) = GetSlideSize();
        long x = (slideW - cx) / 2;
        long y = (slideH - cy) / 2;
        if (properties.TryGetValue("x", out var xs) || properties.TryGetValue("left", out xs)) x = ParseEmu(xs);
        if (properties.TryGetValue("y", out var ys) || properties.TryGetValue("top", out ys)) y = ParseEmu(ys);

        var shapeId = (uint)(shapeTree.ChildElements.Count + 2);
        var shapeName = properties.GetValueOrDefault("name", $"3D Model {shapeId}");

        // Namespaces
        var mcNs = "http://schemas.openxmlformats.org/markup-compatibility/2006";
        var pNs = "http://schemas.openxmlformats.org/presentationml/2006/main";
        var aNs = "http://schemas.openxmlformats.org/drawingml/2006/main";
        var rNs = "http://schemas.openxmlformats.org/officeDocument/2006/relationships";
        var a16Ns = "http://schemas.microsoft.com/office/drawing/2014/main";

        var creationGuid = Guid.NewGuid().ToString("B").ToUpperInvariant();

        // Build mc:AlternateContent
        var acElement = new OpenXmlUnknownElement("mc", "AlternateContent", mcNs);

        // === mc:Choice (for clients that support 3D models) ===
        var choiceElement = new OpenXmlUnknownElement("mc", "Choice", mcNs);
        choiceElement.SetAttribute(new OpenXmlAttribute("", "Requires", null!, "am3d"));
        choiceElement.AddNamespaceDeclaration("am3d", Am3dNs);

        // Use p:graphicFrame (NOT p:sp) — same as zoom and native PowerPoint
        var gf = new OpenXmlUnknownElement("p", "graphicFrame", pNs);
        gf.AddNamespaceDeclaration("a", aNs);
        gf.AddNamespaceDeclaration("r", rNs);

        // nvGraphicFramePr
        var nvGfPr = new OpenXmlUnknownElement("p", "nvGraphicFramePr", pNs);
        var cNvPr = new OpenXmlUnknownElement("p", "cNvPr", pNs);
        cNvPr.SetAttribute(new OpenXmlAttribute("", "id", null!, shapeId.ToString()));
        cNvPr.SetAttribute(new OpenXmlAttribute("", "name", null!, shapeName));
        // creationId extension
        var extLst = new OpenXmlUnknownElement("a", "extLst", aNs);
        var ext = new OpenXmlUnknownElement("a", "ext", aNs);
        ext.SetAttribute(new OpenXmlAttribute("", "uri", null!, "{FF2B5EF4-FFF2-40B4-BE49-F238E27FC236}"));
        var creationId = new OpenXmlUnknownElement("a16", "creationId", a16Ns);
        creationId.SetAttribute(new OpenXmlAttribute("", "id", null!, creationGuid));
        ext.AppendChild(creationId);
        extLst.AppendChild(ext);
        cNvPr.AppendChild(extLst);
        nvGfPr.AppendChild(cNvPr);

        var cNvGfSpPr = new OpenXmlUnknownElement("p", "cNvGraphicFramePr", pNs);
        var gfLocks = new OpenXmlUnknownElement("a", "graphicFrameLocks", aNs);
        gfLocks.SetAttribute(new OpenXmlAttribute("", "noChangeAspect", null!, "1"));
        cNvGfSpPr.AppendChild(gfLocks);
        nvGfPr.AppendChild(cNvGfSpPr);

        nvGfPr.AppendChild(new OpenXmlUnknownElement("p", "nvPr", pNs));
        gf.AppendChild(nvGfPr);

        // xfrm (position/size on the graphicFrame level)
        var gfXfrm = new OpenXmlUnknownElement("p", "xfrm", pNs);
        var gfOff = new OpenXmlUnknownElement("a", "off", aNs);
        gfOff.SetAttribute(new OpenXmlAttribute("", "x", null!, x.ToString()));
        gfOff.SetAttribute(new OpenXmlAttribute("", "y", null!, y.ToString()));
        var gfExt = new OpenXmlUnknownElement("a", "ext", aNs);
        gfExt.SetAttribute(new OpenXmlAttribute("", "cx", null!, cx.ToString()));
        gfExt.SetAttribute(new OpenXmlAttribute("", "cy", null!, cy.ToString()));
        gfXfrm.AppendChild(gfOff);
        gfXfrm.AppendChild(gfExt);
        gf.AppendChild(gfXfrm);

        // a:graphic > a:graphicData[uri=am3d] > am3d:model3d
        var graphic = new OpenXmlUnknownElement("a", "graphic", aNs);
        var graphicData = new OpenXmlUnknownElement("a", "graphicData", aNs);
        graphicData.SetAttribute(new OpenXmlAttribute("", "uri", null!, Am3dNs));

        var model3d = BuildModel3DElement(modelRelId, imageRelId, cx, cy, properties, glbBounds);
        graphicData.AppendChild(model3d);
        graphic.AppendChild(graphicData);
        gf.AppendChild(graphic);

        choiceElement.AppendChild(gf);

        // === mc:Fallback (static image for older clients) ===
        var fallbackElement = new OpenXmlUnknownElement("mc", "Fallback", mcNs);
        var fbPic = new OpenXmlUnknownElement("p", "pic", pNs);
        fbPic.AddNamespaceDeclaration("a", aNs);
        fbPic.AddNamespaceDeclaration("r", rNs);

        var fbNvPicPr = new OpenXmlUnknownElement("p", "nvPicPr", pNs);
        var fbCNvPr = new OpenXmlUnknownElement("p", "cNvPr", pNs);
        fbCNvPr.SetAttribute(new OpenXmlAttribute("", "id", null!, shapeId.ToString()));
        fbCNvPr.SetAttribute(new OpenXmlAttribute("", "name", null!, shapeName));
        // Same creationId
        var fbExtLst = new OpenXmlUnknownElement("a", "extLst", aNs);
        var fbExt = new OpenXmlUnknownElement("a", "ext", aNs);
        fbExt.SetAttribute(new OpenXmlAttribute("", "uri", null!, "{FF2B5EF4-FFF2-40B4-BE49-F238E27FC236}"));
        var fbCreationId = new OpenXmlUnknownElement("a16", "creationId", a16Ns);
        fbCreationId.SetAttribute(new OpenXmlAttribute("", "id", null!, creationGuid));
        fbExt.AppendChild(fbCreationId);
        fbExtLst.AppendChild(fbExt);
        fbCNvPr.AppendChild(fbExtLst);
        fbNvPicPr.AppendChild(fbCNvPr);

        var fbCNvPicPr = new OpenXmlUnknownElement("p", "cNvPicPr", pNs);
        var picLocks = new OpenXmlUnknownElement("a", "picLocks", aNs);
        foreach (var lockAttr in new[] { "noGrp", "noRot", "noChangeAspect", "noMove", "noResize",
            "noEditPoints", "noAdjustHandles", "noChangeArrowheads", "noChangeShapeType", "noCrop" })
            picLocks.SetAttribute(new OpenXmlAttribute("", lockAttr, null!, "1"));
        fbCNvPicPr.AppendChild(picLocks);
        fbNvPicPr.AppendChild(fbCNvPicPr);
        fbNvPicPr.AppendChild(new OpenXmlUnknownElement("p", "nvPr", pNs));
        fbPic.AppendChild(fbNvPicPr);

        // Fallback blipFill
        var fbBlipFill = new OpenXmlUnknownElement("p", "blipFill", pNs);
        var fbBlip = new OpenXmlUnknownElement("a", "blip", aNs);
        fbBlip.SetAttribute(new OpenXmlAttribute("r", "embed", rNs, imageRelId));
        fbBlipFill.AppendChild(fbBlip);
        var fbStretch = new OpenXmlUnknownElement("a", "stretch", aNs);
        fbStretch.AppendChild(new OpenXmlUnknownElement("a", "fillRect", aNs));
        fbBlipFill.AppendChild(fbStretch);
        fbPic.AppendChild(fbBlipFill);

        // Fallback spPr
        var fbSpPr = new OpenXmlUnknownElement("p", "spPr", pNs);
        var fbXfrm = new OpenXmlUnknownElement("a", "xfrm", aNs);
        var fbOff = new OpenXmlUnknownElement("a", "off", aNs);
        fbOff.SetAttribute(new OpenXmlAttribute("", "x", null!, x.ToString()));
        fbOff.SetAttribute(new OpenXmlAttribute("", "y", null!, y.ToString()));
        var fbExtSz = new OpenXmlUnknownElement("a", "ext", aNs);
        fbExtSz.SetAttribute(new OpenXmlAttribute("", "cx", null!, cx.ToString()));
        fbExtSz.SetAttribute(new OpenXmlAttribute("", "cy", null!, cy.ToString()));
        fbXfrm.AppendChild(fbOff);
        fbXfrm.AppendChild(fbExtSz);
        fbSpPr.AppendChild(fbXfrm);
        var fbGeom = new OpenXmlUnknownElement("a", "prstGeom", aNs);
        fbGeom.SetAttribute(new OpenXmlAttribute("", "prst", null!, "rect"));
        fbGeom.AppendChild(new OpenXmlUnknownElement("a", "avLst", aNs));
        fbSpPr.AppendChild(fbGeom);
        fbPic.AppendChild(fbSpPr);

        fallbackElement.AppendChild(fbPic);

        acElement.AppendChild(choiceElement);
        acElement.AppendChild(fallbackElement);
        shapeTree.AppendChild(acElement);

        // Ensure am3d namespace is declared on slide root
        var slide = GetSlide(slidePart);
        try { slide.AddNamespaceDeclaration("am3d", Am3dNs); } catch { }
        try { slide.AddNamespaceDeclaration("mc", mcNs); } catch { }
        var ignorable = slide.MCAttributes?.Ignorable?.Value;
        if (ignorable == null || !ignorable.Contains("am3d"))
        {
            slide.MCAttributes ??= new MarkupCompatibilityAttributes();
            slide.MCAttributes.Ignorable = string.IsNullOrEmpty(ignorable) ? "am3d" : $"{ignorable} am3d";
        }
        slide.Save();

        var model3dCount = GetModel3DElements(shapeTree).Count;
        return $"/slide[{slideIdx}]/model3d[{model3dCount}]";
    }

    /// <summary>
    /// Build the am3d:model3d element with camera, transform, viewport, and lighting.
    /// Follows the native PowerPoint XML structure exactly.
    /// </summary>
    private OpenXmlUnknownElement BuildModel3DElement(
        string modelRelId, string imageRelId, long cx, long cy,
        Dictionary<string, string> properties, GlbBoundingBox bounds)
    {
        var aNs = "http://schemas.openxmlformats.org/drawingml/2006/main";
        var rNs = "http://schemas.openxmlformats.org/officeDocument/2006/relationships";

        var model3d = new OpenXmlUnknownElement("am3d", "model3d", Am3dNs);
        model3d.SetAttribute(new OpenXmlAttribute("r", "embed", rNs, modelRelId));

        // Compute mpu early — needed by camera and trans
        // mpu = mpuFactor / maxExtent, factor chosen by model scale:
        //   tiny (ext < 5):     factor = 0.001 → sun(1)=1000, box(2)=500
        //   medium (5~500):     factor = 100   → duck(165)=604308
        //   large (>= 500):     factor = 1     → saturn(2331)=429, toycar(740)=1352
        // Verified against native PowerPoint for sun, duck, saturn
        double mpuFactor = bounds.MaxExtent < 5 ? 0.001 : bounds.MaxExtent < 500 ? 100.0 : 1.0;
        var mpuVal = bounds.MaxExtent > 0 ? mpuFactor / bounds.MaxExtent : 0.5;

        // 1. spPr (internal shape properties for the 3D model viewport)
        var spPr = new OpenXmlUnknownElement("am3d", "spPr", Am3dNs);
        var xfrm = new OpenXmlUnknownElement("a", "xfrm", aNs);
        var off = new OpenXmlUnknownElement("a", "off", aNs);
        off.SetAttribute(new OpenXmlAttribute("", "x", null!, "0"));
        off.SetAttribute(new OpenXmlAttribute("", "y", null!, "0"));
        var ext = new OpenXmlUnknownElement("a", "ext", aNs);
        ext.SetAttribute(new OpenXmlAttribute("", "cx", null!, cx.ToString()));
        ext.SetAttribute(new OpenXmlAttribute("", "cy", null!, cy.ToString()));
        xfrm.AppendChild(off);
        xfrm.AppendChild(ext);
        spPr.AppendChild(xfrm);
        var prstGeom = new OpenXmlUnknownElement("a", "prstGeom", aNs);
        prstGeom.SetAttribute(new OpenXmlAttribute("", "prst", null!, "rect"));
        prstGeom.AppendChild(new OpenXmlUnknownElement("a", "avLst", aNs));
        spPr.AppendChild(prstGeom);
        model3d.AppendChild(spPr);

        // 2. camera — perspective, looking at origin from z-axis
        // Camera Z ≈ 70000000 (constant, matches native PowerPoint for all models)
        // viewportSz ≈ max(cx,cy) * 1.5
        var viewportSize = (long)(Math.Max(cx, cy) * 1.5);
        var defaultCamZ = "70000000";
        var camPosX = properties.GetValueOrDefault("camerax", "0");
        var camPosY = properties.GetValueOrDefault("cameray", "0");
        var camPosZ = properties.GetValueOrDefault("cameraz", defaultCamZ);

        var camera = new OpenXmlUnknownElement("am3d", "camera", Am3dNs);
        var camPos = new OpenXmlUnknownElement("am3d", "pos", Am3dNs);
        camPos.SetAttribute(new OpenXmlAttribute("", "x", null!, camPosX));
        camPos.SetAttribute(new OpenXmlAttribute("", "y", null!, camPosY));
        camPos.SetAttribute(new OpenXmlAttribute("", "z", null!, camPosZ));
        camera.AppendChild(camPos);
        var camUp = new OpenXmlUnknownElement("am3d", "up", Am3dNs);
        camUp.SetAttribute(new OpenXmlAttribute("", "dx", null!, "0"));
        camUp.SetAttribute(new OpenXmlAttribute("", "dy", null!, "36000000"));
        camUp.SetAttribute(new OpenXmlAttribute("", "dz", null!, "0"));
        camera.AppendChild(camUp);
        var camLookAt = new OpenXmlUnknownElement("am3d", "lookAt", Am3dNs);
        camLookAt.SetAttribute(new OpenXmlAttribute("", "x", null!, "0"));
        camLookAt.SetAttribute(new OpenXmlAttribute("", "y", null!, "0"));
        camLookAt.SetAttribute(new OpenXmlAttribute("", "z", null!, "0"));
        camera.AppendChild(camLookAt);
        var perspective = new OpenXmlUnknownElement("am3d", "perspective", Am3dNs);
        perspective.SetAttribute(new OpenXmlAttribute("", "fov", null!, "2700000")); // 45 degrees
        camera.AppendChild(perspective);
        model3d.AppendChild(camera);

        // 3. trans (model transform) — computed from GLB bounding box
        // mpu = 1 / maxExtent
        var trans = new OpenXmlUnknownElement("am3d", "trans", Am3dNs);
        var mpuN = (long)(mpuVal * 1000000);
        var mpu = new OpenXmlUnknownElement("am3d", "meterPerModelUnit", Am3dNs);
        mpu.SetAttribute(new OpenXmlAttribute("", "n", null!, mpuN.ToString()));
        mpu.SetAttribute(new OpenXmlAttribute("", "d", null!, "1000000"));
        trans.AppendChild(mpu);

        // preTrans: shift model center to origin = -center * mpu * 360000
        // Must be coupled with mpu so the shift is proportional to model size
        var preTransScale = mpuVal * 360000.0;
        var preTrans = new OpenXmlUnknownElement("am3d", "preTrans", Am3dNs);
        preTrans.SetAttribute(new OpenXmlAttribute("", "dx", null!, ((long)(-bounds.CenterX * preTransScale)).ToString()));
        preTrans.SetAttribute(new OpenXmlAttribute("", "dy", null!, ((long)(-bounds.CenterY * preTransScale)).ToString()));
        preTrans.SetAttribute(new OpenXmlAttribute("", "dz", null!, ((long)(-bounds.CenterZ * preTransScale)).ToString()));
        trans.AppendChild(preTrans);

        // scale (default 1:1:1)
        var scale = new OpenXmlUnknownElement("am3d", "scale", Am3dNs);
        foreach (var axis in new[] { "sx", "sy", "sz" })
        {
            var s = new OpenXmlUnknownElement("am3d", axis, Am3dNs);
            s.SetAttribute(new OpenXmlAttribute("", "n", null!, "1000000"));
            s.SetAttribute(new OpenXmlAttribute("", "d", null!, "1000000"));
            scale.AppendChild(s);
        }
        trans.AppendChild(scale);

        // rot
        var rot = new OpenXmlUnknownElement("am3d", "rot", Am3dNs);
        var rotXVal = "0"; var rotYVal = "0"; var rotZVal = "0";
        if (properties.TryGetValue("rotx", out var rx)) rotXVal = ParseAngle60k(rx).ToString();
        if (properties.TryGetValue("roty", out var ry)) rotYVal = ParseAngle60k(ry).ToString();
        if (properties.TryGetValue("rotz", out var rz)) rotZVal = ParseAngle60k(rz).ToString();
        rot.SetAttribute(new OpenXmlAttribute("", "ax", null!, rotXVal));
        rot.SetAttribute(new OpenXmlAttribute("", "ay", null!, rotYVal));
        rot.SetAttribute(new OpenXmlAttribute("", "az", null!, rotZVal));
        trans.AppendChild(rot);

        // postTrans
        var postTrans = new OpenXmlUnknownElement("am3d", "postTrans", Am3dNs);
        postTrans.SetAttribute(new OpenXmlAttribute("", "dx", null!, "0"));
        postTrans.SetAttribute(new OpenXmlAttribute("", "dy", null!, "0"));
        postTrans.SetAttribute(new OpenXmlAttribute("", "dz", null!, "0"));
        trans.AppendChild(postTrans);

        model3d.AppendChild(trans);

        // 4. raster (cached rendering) — use am3d:blip (not a:blip)
        var raster = new OpenXmlUnknownElement("am3d", "raster", Am3dNs);
        raster.SetAttribute(new OpenXmlAttribute("", "rName", null!, "Office3DRenderer"));
        raster.SetAttribute(new OpenXmlAttribute("", "rVer", null!, "16.0.8326"));
        var rasterBlip = new OpenXmlUnknownElement("am3d", "blip", Am3dNs);
        rasterBlip.SetAttribute(new OpenXmlAttribute("r", "embed", rNs, imageRelId));
        raster.AppendChild(rasterBlip);
        model3d.AppendChild(raster);

        // 5. objViewport — matches the shape size
        var viewport = new OpenXmlUnknownElement("am3d", "objViewport", Am3dNs);
        viewport.SetAttribute(new OpenXmlAttribute("", "viewportSz", null!, viewportSize.ToString()));
        model3d.AppendChild(viewport);

        // 6. ambientLight — use scrgbClr like native PowerPoint
        var ambient = new OpenXmlUnknownElement("am3d", "ambientLight", Am3dNs);
        var ambClr = new OpenXmlUnknownElement("am3d", "clr", Am3dNs);
        var ambScrgb = new OpenXmlUnknownElement("a", "scrgbClr", aNs);
        ambScrgb.SetAttribute(new OpenXmlAttribute("", "r", null!, "50000"));
        ambScrgb.SetAttribute(new OpenXmlAttribute("", "g", null!, "50000"));
        ambScrgb.SetAttribute(new OpenXmlAttribute("", "b", null!, "50000"));
        ambClr.AppendChild(ambScrgb);
        ambient.AppendChild(ambClr);
        var ambIll = new OpenXmlUnknownElement("am3d", "illuminance", Am3dNs);
        ambIll.SetAttribute(new OpenXmlAttribute("", "n", null!, "500000"));
        ambIll.SetAttribute(new OpenXmlAttribute("", "d", null!, "1000000"));
        ambient.AppendChild(ambIll);
        model3d.AppendChild(ambient);

        // 7. ptLight — three point lights (matching native PowerPoint)
        AddPointLight(model3d, aNs, "100000", "75000", "50000", "9765625", "21959998", "70920001", "16344003");
        AddPointLight(model3d, aNs, "40000", "60000", "95000", "12250000", "-37964106", "51130435", "57631972");
        AddPointLight(model3d, aNs, "86837", "72700", "100000", "3125000", "-37739122", "58056624", "-34769649");

        return model3d;
    }

    private static void AddPointLight(OpenXmlUnknownElement parent, string aNs,
        string r, string g, string b, string intensity,
        string posX, string posY, string posZ)
    {
        var ptLight = new OpenXmlUnknownElement("am3d", "ptLight", Am3dNs);
        ptLight.SetAttribute(new OpenXmlAttribute("", "rad", null!, "0"));
        var ptClr = new OpenXmlUnknownElement("am3d", "clr", Am3dNs);
        var ptScrgb = new OpenXmlUnknownElement("a", "scrgbClr", aNs);
        ptScrgb.SetAttribute(new OpenXmlAttribute("", "r", null!, r));
        ptScrgb.SetAttribute(new OpenXmlAttribute("", "g", null!, g));
        ptScrgb.SetAttribute(new OpenXmlAttribute("", "b", null!, b));
        ptClr.AppendChild(ptScrgb);
        ptLight.AppendChild(ptClr);
        var ptInt = new OpenXmlUnknownElement("am3d", "intensity", Am3dNs);
        ptInt.SetAttribute(new OpenXmlAttribute("", "n", null!, intensity));
        ptInt.SetAttribute(new OpenXmlAttribute("", "d", null!, "1000000"));
        ptLight.AppendChild(ptInt);
        var ptPos = new OpenXmlUnknownElement("am3d", "pos", Am3dNs);
        ptPos.SetAttribute(new OpenXmlAttribute("", "x", null!, posX));
        ptPos.SetAttribute(new OpenXmlAttribute("", "y", null!, posY));
        ptPos.SetAttribute(new OpenXmlAttribute("", "z", null!, posZ));
        ptLight.AppendChild(ptPos);
        parent.AppendChild(ptLight);
    }

    /// <summary>
    /// Parse degrees to 60000ths-of-a-degree for am3d rotation attributes.
    /// </summary>
    private static int ParseAngle60k(string value)
    {
        if (!double.TryParse(value, System.Globalization.NumberStyles.Float,
            System.Globalization.CultureInfo.InvariantCulture, out var deg))
            return 0;
        return (int)(deg * 60000);
    }

    /// <summary>
    /// Bounding box info extracted from a GLB file.
    /// </summary>
    private record GlbBoundingBox(
        double CenterX, double CenterY, double CenterZ,
        double ExtentX, double ExtentY, double ExtentZ,
        double MaxExtent, double MeterPerModelUnit);

    /// <summary>
    /// Parse a GLB file to extract the bounding box from accessor min/max values.
    /// Used to compute preTrans (centering) and meterPerModelUnit (scaling).
    /// </summary>
    private static GlbBoundingBox ParseGlbBoundingBox(string glbPath)
    {
        try
        {
            using var fs = File.OpenRead(glbPath);
            using var reader = new BinaryReader(fs);

            // GLB header: magic(4) + version(4) + length(4)
            var magic = reader.ReadUInt32(); // 0x46546C67 = "glTF"
            var version = reader.ReadUInt32();
            var totalLen = reader.ReadUInt32();

            // JSON chunk: length(4) + type(4) + data
            var chunkLen = reader.ReadUInt32();
            var chunkType = reader.ReadUInt32(); // 0x4E4F534A = "JSON"
            var jsonBytes = reader.ReadBytes((int)chunkLen);
            var json = System.Text.Encoding.UTF8.GetString(jsonBytes);

            // Parse accessors to find position min/max
            double gMinX = double.MaxValue, gMinY = double.MaxValue, gMinZ = double.MaxValue;
            double gMaxX = double.MinValue, gMaxY = double.MinValue, gMaxZ = double.MinValue;
            bool found = false;

            // Simple JSON parsing for "min":[x,y,z] and "max":[x,y,z] in accessors
            var doc = System.Text.Json.JsonDocument.Parse(json);
            if (doc.RootElement.TryGetProperty("accessors", out var accessors))
            {
                foreach (var acc in accessors.EnumerateArray())
                {
                    if (acc.TryGetProperty("min", out var min) &&
                        acc.TryGetProperty("max", out var max) &&
                        min.GetArrayLength() == 3 && max.GetArrayLength() == 3)
                    {
                        found = true;
                        var mnX = min[0].GetDouble(); var mnY = min[1].GetDouble(); var mnZ = min[2].GetDouble();
                        var mxX = max[0].GetDouble(); var mxY = max[1].GetDouble(); var mxZ = max[2].GetDouble();
                        if (mnX < gMinX) gMinX = mnX; if (mnY < gMinY) gMinY = mnY; if (mnZ < gMinZ) gMinZ = mnZ;
                        if (mxX > gMaxX) gMaxX = mxX; if (mxY > gMaxY) gMaxY = mxY; if (mxZ > gMaxZ) gMaxZ = mxZ;
                    }
                }
            }

            if (!found)
                return new GlbBoundingBox(0, 0, 0, 1, 1, 1, 1, 0.5);

            var cx = (gMinX + gMaxX) / 2;
            var cy = (gMinY + gMaxY) / 2;
            var cz = (gMinZ + gMaxZ) / 2;
            var ex = gMaxX - gMinX;
            var ey = gMaxY - gMinY;
            var ez = gMaxZ - gMinZ;
            var maxExt = Math.Max(ex, Math.Max(ey, ez));
            var mpu = maxExt > 0 ? 1.0 / (2.0 * maxExt) : 0.5;

            return new GlbBoundingBox(cx, cy, cz, ex, ey, ez, maxExt, mpu);
        }
        catch
        {
            // Fallback for unparseable files
            return new GlbBoundingBox(0, 0, 0, 1, 1, 1, 1, 0.5);
        }
    }

}
