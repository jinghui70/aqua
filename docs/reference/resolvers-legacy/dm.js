/**
 * DM (达梦) resolver — maps DM JDBC raw physical types to aqua logical DataType.
 * Similar to Oracle lineage.
 */

module.exports = {
  type: 'dm',

  resolve: function (column) {
    var raw = column.rawType.toUpperCase();
    var length = column.length;
    var precision = column.precision;
    var scale = column.scale;

    // VARCHAR / VARCHAR2 → VARCHAR(n)
    if (raw === 'VARCHAR' || raw === 'VARCHAR2') {
      return { dataType: 'VARCHAR', length: length > 0 ? length : undefined };
    }

    // CHAR → VARCHAR(n)
    if (raw === 'CHAR') {
      return { dataType: 'VARCHAR', length: length > 0 ? length : undefined };
    }

    // CLOB → CLOB
    if (raw === 'CLOB') {
      return { dataType: 'CLOB' };
    }

    // TINYINT → TINYINT
    if (raw === 'TINYINT') {
      return { dataType: 'TINYINT' };
    }

    // INT → INT
    if (raw === 'INT') {
      return { dataType: 'INT' };
    }

    // BIGINT → LONG
    if (raw === 'BIGINT') {
      return { dataType: 'LONG' };
    }

    // DECIMAL / NUMBER → DECIMAL(p,s)
    if (raw === 'DECIMAL' || raw === 'NUMBER') {
      var p = precision != null && precision > 0 ? precision : undefined;
      var s = scale != null && scale >= 0 ? scale : undefined;
      // If no scale, it's an integer→INT/LONG
      if (s == null || s === 0) {
        if (p != null && p > 10) {
          return { dataType: 'LONG' };
        }
        return { dataType: 'INT' };
      }
      return { dataType: 'DECIMAL', precision: p, scale: s };
    }

    // DATE → DATE
    if (raw === 'DATE') {
      return { dataType: 'DATE' };
    }

    // TIMESTAMP → DATETIME
    if (raw.indexOf('TIMESTAMP') === 0) {
      return { dataType: 'DATETIME' };
    }

    // BLOB / BINARY → BLOB
    if (raw === 'BLOB' || raw === 'BINARY' || raw === 'IMAGE') {
      return { dataType: 'BLOB' };
    }

    return null;
  }
};
